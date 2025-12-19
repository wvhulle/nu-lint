use nu_protocol::{
    Span,
    ast::{Block, Expr, Expression, Pipeline, Traverse},
};

use crate::{
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    effect::external::is_external_command_safe,
    rule::Rule,
    violation::Violation,
};

fn is_wrapped_in_complete(pipeline: &Pipeline, context: &LintContext) -> bool {
    pipeline.elements.iter().any(|element| {
        matches!(&element.expr.expr, Expr::Call(call)
            if call.is_call_to_command("complete", context))
    })
}

fn is_in_try_block(expr_span: Span, context: &LintContext) -> bool {
    use nu_protocol::ast::Traverse;

    let mut try_spans = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            matches!(&expr.expr, Expr::Call(call)
            if call.is_call_to_command("try", context))
            .then_some(expr.span)
            .into_iter()
            .collect()
        },
        &mut try_spans,
    );

    try_spans
        .iter()
        .any(|try_span| try_span.contains_span(expr_span))
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Option<Violation> {
    if !{ pipeline.elements.len() > 1 } {
        return None;
    }

    for (i, element) in pipeline.elements[0..pipeline.elements.len() - 1]
        .iter()
        .enumerate()
    {
        if let Expr::ExternalCall(command, _) = &element.expr.expr {
            let external_command = command.span_text(context);
            log::debug!("Found an external call to {external_command} in the pipeline");
            if !is_external_command_safe(external_command) {
                log::debug!("External call to {external_command} is not safe");
                if i + 1 > pipeline.elements.len() - 1 {
                    log::debug!("This is already the last element.");
                    return None;
                }
                let next_pipeline_element = &pipeline.elements[i + 1].expr.expr;

                if let Expr::Call(call) = &next_pipeline_element {
                    let builtin_name = call.get_call_name(context);
                    log::debug!("Next element is a call to built-in {builtin_name}.");
                    if call.is_call_to_command("complete", context) {
                        return None;
                    }
                    let message = format!(
                        "External command '{external_command}' in pipeline without error handling: Nushell only \
         checks the last command's exit code. If this command fails, the error will be silently \
         ignored."
                    );

                    let help = "Wrap the external command in 'complete' to capture its exit code.";

                    let last_element_span = pipeline.elements.last().map(|e| e.expr.span);

                    let mut violation = Violation::new(message, element.expr.span)
                        .with_primary_label("external command without error handling");

                    if let Some(last_span) = last_element_span {
                        violation = violation.with_extra_label(
                            "only this command's exit code is checked",
                            last_span,
                        );
                    }

                    return Some(violation.with_help(help));
                }
            }
        }
    }

    None
}

fn check_block(block: &Block, context: &LintContext, violations: &mut Vec<Violation>) {
    for pipeline in &block.pipelines {
        violations.extend(check_pipeline(pipeline, context));

        for element in &pipeline.elements {
            let mut blocks = Vec::new();
            element.expr.flat_map(
                context.working_set,
                &|expr| match &expr.expr {
                    Expr::Block(block_id)
                    | Expr::Closure(block_id)
                    | Expr::Subexpression(block_id) => {
                        vec![*block_id]
                    }
                    _ => vec![],
                },
                &mut blocks,
            );

            for &block_id in &blocks {
                let nested_block = context.working_set.get_block(block_id);
                check_block(nested_block, context, violations);
            }
        }
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    let mut violations = Vec::new();
    check_block(context.ast, context, &mut violations);
    violations
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_complete_for_external_commands",
        "External commands in pipelines should use 'complete' for error handling (Nushell doesn't \
         propagate pipeline errors by default)",
        check,
    )
    .with_doc_url("https://www.nushell.sh/commands/docs/complete.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
