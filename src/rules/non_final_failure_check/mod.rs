use nu_protocol::ast::{self, Block, Expr, Pipeline, Traverse};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    effect::external::is_external_command_safe,
    rule::Rule,
    violation::Violation,
};

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Option<Violation> {
    if pipeline.elements.len() == 1 {
        return None;
    }

    for (i, element) in pipeline.elements[0..pipeline.elements.len() - 1]
        .iter()
        .enumerate()
    {
        if let Expr::ExternalCall(command, _) = &element.expr.expr {
            let external_command = command.span_text(context);
            log::debug!(
                "Found an external call to {external_command} in the pipeline at position {i}."
            );
            if is_external_command_safe(external_command) {
                continue;
            }
            log::debug!("External call to {external_command} is not safe");

            let next_pipeline_element = &pipeline.elements[i + 1].expr.expr;

            if let Expr::Call(call) = &next_pipeline_element
                && call.is_call_to_command("complete", context)
            {
                continue;
            }
            let violation = create_violation(pipeline, element, external_command);
            return Some(violation);
        }
    }

    None
}

fn create_violation(
    pipeline: &Pipeline,
    element: &ast::PipelineElement,
    external_command: &str,
) -> Violation {
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
        violation =
            violation.with_extra_label("only this command's exit code is checked", last_span);
    }

    violation.with_help(help)
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

pub const RULE: Rule = Rule::new(
    "non_final_failure_check",
    "Only the exit code of the last external command in a pipeline is reported.",
    check,
    LintLevel::Warning,
)
.with_doc_url("https://www.nushell.sh/blog/2025-10-15-nushell_v0_108_0.html#pipefail-16449-toc");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
