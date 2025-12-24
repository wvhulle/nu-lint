use nu_protocol::ast::{Block, Expr, ExternalArgument, PipelineElement};

use crate::{
    ast::block::BlockExt, config::LintLevel, context::LintContext, rule::Rule, violation::Violation,
};

fn is_nu_with_c_flag(cmd_name: &str, args: &[ExternalArgument], context: &LintContext) -> bool {
    if cmd_name != "nu" {
        return false;
    }

    args.iter().any(|arg| {
        let ExternalArgument::Regular(expr) = arg else {
            return false;
        };
        let arg_text = context.get_span_text(expr.span);
        arg_text == "-c" || arg_text == "--commands"
    })
}

fn check_element(element: &PipelineElement, context: &LintContext) -> Option<Violation> {
    let Expr::ExternalCall(head, args) = &element.expr.expr else {
        return None;
    };

    let cmd_name = context.get_span_text(head.span);

    if !is_nu_with_c_flag(cmd_name, args, context) {
        return None;
    }

    Some(
        Violation::new(
            "Avoid spawning `nu -c` subprocess from within a Nu script",
            element.expr.span,
        )
        .with_primary_label("subprocess spawned here")
        .with_help(
            "You're already running Nu. Call functions directly instead of spawning a subprocess. \
             Use `match` or direct function calls to dispatch to subcommands.",
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
    "avoid_nu_subprocess",
    "Spawning `nu -c` from within a Nu script is redundant; call functions directly instead",
    check,
    LintLevel::Error,
);

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
