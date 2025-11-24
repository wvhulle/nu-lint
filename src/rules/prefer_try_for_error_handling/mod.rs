use nu_protocol::{
    Span,
    ast::{Expr, Expression, FindMapResult, Traverse},
};

use crate::{
    ast::{
        call::CallExt,
        effect::{SideEffect, can_error, has_external_side_effect},
    },
    context::LintContext,
    rule::Rule,
    violation::Violation,
};

fn has_external_command(expr: &Expression, context: &LintContext) -> bool {
    expr.find_map(context.working_set, &|inner_expr| {
        if let Expr::ExternalCall(head, args) = &inner_expr.expr {
            let cmd_name = &context.source[head.span.start..head.span.end];
            // External commands can error unless explicitly marked otherwise
            // Check if we have explicit info, otherwise assume it can error
            if has_external_side_effect(cmd_name, SideEffect::MayErrorFrequently, context, args) {
                return FindMapResult::Found(());
            }
            // If not in registry, assume external commands can error (conservative
            // approach)
            return FindMapResult::Found(());
        }
        FindMapResult::Continue
    })
    .is_some()
}

fn has_error_prone_builtin(expr: &Expression, context: &LintContext) -> bool {
    expr.find_map(context.working_set, &|inner_expr| {
        if let Expr::Call(call) = &inner_expr.expr {
            let cmd_name = call.get_call_name(context);
            log::debug!("Checking command: {cmd_name}");
            if can_error(&cmd_name, context, call) {
                log::debug!("Found error-prone builtin command: {cmd_name}");
                return FindMapResult::Found(());
            }
        }
        FindMapResult::Continue
    })
    .is_some()
}

fn is_do_block_with_error_prone_ops(expr: &Expression, context: &LintContext) -> Option<Span> {
    if let Expr::Call(call) = &expr.expr
        && call.is_call_to_command("do", context)
        && let Some(block_arg) = call.get_positional_arg(0)
    {
        log::debug!("Found do block, checking for error-prone operations");
        let has_external = has_external_command(block_arg, context);
        let has_builtin = has_error_prone_builtin(block_arg, context);
        log::debug!("External commands: {has_external}, Error-prone builtins: {has_builtin}");
        if has_external || has_builtin {
            log::debug!(
                "Found do block with error-prone operations at span {:?}",
                expr.span
            );
            return Some(expr.span);
        }
    }
    None
}

fn check(context: &LintContext) -> Vec<Violation> {
    let mut violations = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            is_do_block_with_error_prone_ops(expr, context).map_or_else(Vec::new, |span| {
                vec![
                    Violation::new(
                        "prefer_try_for_error_handling",
                        "Use 'try' blocks instead of 'do' blocks for error-prone operations"
                            .to_string(),
                        span,
                    )
                    .with_help(
                        "Replace 'do { ... }' with 'try { ... }' when the block contains external \
                         commands or error-prone operations like file I/O or network requests. \
                         This provides proper error handling and prevents script termination on \
                         failures.",
                    ),
                ]
            })
        },
        &mut violations,
    );

    violations
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_try_for_error_handling",
        "Use 'try' blocks instead of 'do' blocks for error-prone operations",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
