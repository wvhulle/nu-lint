use nu_protocol::ast::{Argument, Block, Expr, PipelineElement};

use crate::{ast::call::CallExt, context::LintContext, rule::Rule, violation::Violation};

fn check_sequential_stderr_exit(
    first: &PipelineElement,
    second: &PipelineElement,
    context: &LintContext,
) -> Option<Violation> {
    let print_call = match &first.expr.expr {
        Expr::Call(call) if call.is_call_to_command("print", context) => call,
        _ => return None,
    };

    let has_stderr_flag = print_call.arguments.iter().any(|arg| {
        matches!(arg, Argument::Named(named)
            if named.0.item == "stderr")
    });

    if !has_stderr_flag {
        return None;
    }

    let exit_call = match &second.expr.expr {
        Expr::Call(call) if call.is_call_to_command("exit", context) => call,
        _ => return None,
    };

    Some(
        Violation::new(
            "prefer_error_make_for_stderr",
            "Use 'error make' instead of 'print stderr' + 'exit' for error conditions",
            print_call.span().merge(exit_call.span()),
        )
        .with_help(
            "Use 'error make { msg: \"error message\" }' instead. Consider adding 'label' with \
             span for precise error location, and 'help' field for user guidance.",
        ),
    )
}

fn check_block_pipelines<'a>(
    block: &'a Block,
    context: &'a LintContext<'a>,
) -> impl Iterator<Item = Violation> + 'a {
    block.pipelines.windows(2).filter_map(move |pipelines| {
        let [first_pipeline, second_pipeline] = pipelines else {
            return None;
        };

        let [first_elem] = &first_pipeline.elements[..] else {
            return None;
        };
        let [second_elem] = &second_pipeline.elements[..] else {
            return None;
        };

        check_sequential_stderr_exit(first_elem, second_elem, context)
    })
}

fn check(context: &LintContext) -> Vec<Violation> {
    let main_violations = check_block_pipelines(context.ast, context);

    let nested_violations = context.collect_rule_violations(|expr, ctx| match &expr.expr {
        Expr::Closure(block_id) | Expr::Block(block_id) => {
            let block = ctx.working_set.get_block(*block_id);
            check_block_pipelines(block, ctx).collect()
        }
        _ => vec![],
    });

    main_violations.chain(nested_violations).collect()
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_error_make_for_stderr",
        "Use 'error make' instead of 'print stderr' + 'exit' for structured error handling",
        check,
    )
    .with_doc_url("https://www.nushell.sh/commands/docs/error_make.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
