use nu_protocol::ast::{Argument, Block, Expr, PipelineElement};

use crate::{
    ast::{block::BlockExt, call::CallExt},
    config::LintLevel,
    context::LintContext,
    rule::Rule,
    violation::Violation,
};

fn contains_current_file_reference(text: &str) -> bool {
    text.contains("$env.CURRENT_FILE")
        || text.contains("$nu.current-file")
        || text.contains("(path self)")
}

fn check_element(element: &PipelineElement, context: &LintContext) -> Option<Violation> {
    let Expr::Call(call) = &element.expr.expr else {
        return None;
    };

    if !call.is_call_to_command("use", context) && !call.is_call_to_command("source", context) {
        return None;
    }

    let has_self_reference = call.arguments.iter().any(|arg| {
        let expr = match arg {
            Argument::Positional(e)
            | Argument::Unknown(e)
            | Argument::Spread(e)
            | Argument::Named((_, _, Some(e))) => e,
            Argument::Named(_) => return false,
        };

        let arg_text = context.get_span_text(expr.span);
        contains_current_file_reference(arg_text)
    });

    if !has_self_reference {
        return None;
    }

    let command_name = call.get_call_name(context);

    Some(
        Violation::new(
            format!("Avoid `{command_name}` with reference to current file (self-import pattern)"),
            element.expr.span,
        )
        .with_primary_label("self-import here")
        .with_help(
            "Self-importing is unnecessary. Functions defined in the same file are already \
             available. Use direct function calls or `match` expressions to dispatch to \
             subcommands in `main`.",
        ),
    )
}

fn check_block(block: &Block, context: &LintContext) -> Vec<Violation> {
    block
        .all_elements()
        .into_iter()
        .filter_map(|element| {
            check_element(element, context).or_else(|| {
                extract_nested_blocks(element, context)
                    .into_iter()
                    .flat_map(|block_id| {
                        let nested_block = context.working_set.get_block(block_id);
                        check_block(nested_block, context)
                    })
                    .next()
            })
        })
        .collect()
}

fn extract_nested_blocks(
    element: &PipelineElement,
    context: &LintContext,
) -> Vec<nu_protocol::BlockId> {
    use nu_protocol::ast::Traverse;

    let mut blocks = Vec::new();
    element.expr.flat_map(
        context.working_set,
        &|expr| match &expr.expr {
            Expr::Block(block_id) | Expr::Closure(block_id) | Expr::Subexpression(block_id) => {
                vec![*block_id]
            }
            _ => vec![],
        },
        &mut blocks,
    );
    blocks
}

fn check(context: &LintContext) -> Vec<Violation> {
    check_block(context.ast, context)
}

pub const RULE: Rule = Rule::new(
    "avoid_self_import",
    "Avoid importing the current script from itself; functions are already available in scope",
    check,
    LintLevel::Error,
);

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
