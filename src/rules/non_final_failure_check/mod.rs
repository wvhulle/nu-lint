use nu_protocol::ast::{self, Block, Expr, Pipeline, Traverse};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    effect::{
        CommonEffect,
        external::{ExternEffect, has_external_side_effect},
    },
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Option<Detection> {
    if pipeline.elements.len() == 1 {
        return None;
    }

    for (i, element) in pipeline.elements[0..pipeline.elements.len() - 1]
        .iter()
        .enumerate()
    {
        if let Expr::ExternalCall(command, external_arguments) = &element.expr.expr {
            let external_command = command.span_text(context);
            log::debug!(
                "Found an external call to {external_command} in the pipeline at position {i}."
            );
            if !has_external_side_effect(
                external_command,
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                context,
                external_arguments,
            ) {
                continue;
            }
            log::debug!("External call to {external_command} is not safe");

            let next_pipeline_element = &pipeline.elements[i + 1].expr.expr;

            if let Expr::Call(call) = &next_pipeline_element
                && call.is_call_to_command("complete", context)
            {
                continue;
            }
            let violation = create_violation(pipeline, element);
            return Some(violation);
        }
    }

    None
}

fn create_violation(pipeline: &Pipeline, element: &ast::PipelineElement) -> Detection {
    let message =
        "Nushell only checks the final external command's exit code in pipelines. ".to_string();

    let last_element_span = pipeline.elements.last().map(|e| e.expr.span);

    let mut violation = Detection::from_global_span(message, element.expr.span)
        .with_primary_label("If this command fails, the error will be silently ignored.");

    if let Some(last_span) = last_element_span {
        violation =
            violation.with_extra_label("only this command's exit code is checked", last_span);
    }

    violation
}

fn check_block(block: &Block, context: &LintContext, violations: &mut Vec<Detection>) {
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

struct NonFinalFailureCheck;

impl DetectFix for NonFinalFailureCheck {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "non_final_failure_check"
    }

    fn short_description(&self) -> &'static str {
        "Only the exit code of the last external command in a pipeline is reported."
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/blog/2025-10-15-nushell_v0_108_0.html#pipefail-16449-toc")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut violations = Vec::new();
        check_block(context.ast, context, &mut violations);
        Self::no_fix(violations)
    }
}

pub static RULE: &dyn Rule = &NonFinalFailureCheck;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
