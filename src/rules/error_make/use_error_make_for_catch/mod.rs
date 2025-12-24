use std::collections::HashSet;

use nu_protocol::{
    BlockId,
    ast::{Argument, Call, Expr, Expression, Traverse},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt, span::SpanExt},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement, Violation},
};

fn collect_try_blocks(context: &LintContext) -> HashSet<BlockId> {
    let mut try_block_ids = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr: &Expression| {
            let Expr::Call(call) = &expr.expr else {
                return vec![];
            };
            if call.is_call_to_command("try", context) {
                call.arguments
                    .iter()
                    .filter_map(|arg| {
                        arg.expr().and_then(|e| match &e.expr {
                            Expr::Block(block_id) => Some(*block_id),
                            _ => None,
                        })
                    })
                    .collect()
            } else {
                vec![]
            }
        },
        &mut try_block_ids,
    );
    try_block_ids.into_iter().collect()
}

fn extract_message_text(call: &Call, ctx: &LintContext) -> Option<String> {
    let msg_expr = call.get_first_positional_arg()?;
    let text = msg_expr.span_text(ctx);

    Some(match &msg_expr.expr {
        Expr::String(s) => format!("\"{s}\""),
        Expr::GlobPattern(_, _) | Expr::RawString(_) => format!("\"{text}\""),
        _ => text.to_string(),
    })
}

fn generate_fix(call: &Call, ctx: &LintContext) -> Option<Fix> {
    let msg_text = extract_message_text(call, ctx)?;
    let replacement = format!("error make {{ msg: {msg_text} }}");

    Some(Fix::with_explanation(
        "Replace with 'error make'",
        vec![Replacement::new(call.span(), replacement)],
    ))
}

fn check(context: &LintContext) -> Vec<Violation> {
    let functions = context.collect_function_definitions();
    let try_blocks = collect_try_blocks(context);

    context.collect_rule_violations(|expr, ctx| {
        let Expr::Call(call) = &expr.expr else {
            return vec![];
        };

        if !call.is_call_to_command("print", ctx) {
            return vec![];
        }

        let has_stderr_flag = call
            .arguments
            .iter()
            .any(|arg| matches!(arg, Argument::Named(named) if named.0.item == "stderr"));

        if !has_stderr_flag {
            return vec![];
        }

        let span = call.head;

        let in_non_main_function = span
            .find_containing_function(&functions, ctx)
            .is_some_and(|name| name != "main");

        let in_try_block = try_blocks.iter().any(|block_id| {
            ctx.working_set
                .get_block(*block_id)
                .span
                .is_some_and(|s| s.contains_span(span))
        });

        if !in_non_main_function && !in_try_block {
            return vec![];
        }

        let context_hint = if in_try_block {
            "inside try block"
        } else {
            "inside reusable function"
        };

        let mut violation = Violation::new(
            format!("Consider 'error make' instead of 'print --stderr' {context_hint}"),
            expr.span,
        )
        .with_primary_label("stderr output in catchable context")
        .with_help(
            "Use 'error make { msg: \"...\" }' to throw a catchable exception. This allows \
             callers to handle the error with 'try/catch'. Reserve 'print --stderr' + 'exit' for \
             top-level unrecoverable termination in 'main'.",
        );

        if let Some(fix) = generate_fix(call, ctx) {
            violation = violation.with_fix(fix);
        }

        vec![violation]
    })
}

pub const RULE: Rule = Rule::new(
    "use_error_make_for_catch",
    "Use 'error make' for catchable errors in functions and try blocks",
    check,
    LintLevel::Hint,
)
.with_auto_fix()
.with_doc_url("https://www.nushell.sh/book/control_flow.html#error-make");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
