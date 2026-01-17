use nu_protocol::ast::{self, Expr, Pipeline};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt, expression::ExpressionExt},
    context::LintContext,
    effect::{
        CommonEffect,
        external::{ExternEffect, has_external_side_effect},
    },
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<Detection> {
    if pipeline.elements.len() == 1 {
        return vec![];
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
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                context,
                external_arguments,
            ) {
                continue;
            }
            log::debug!("External call to {external_command} is not safe");

            let next_pipeline_element = &pipeline.elements[i + 1].expr.expr;

            // Skip if piped to `complete` (captures exit code) or `ignore` (intentional
            // discard)
            if let Expr::Call(call) = &next_pipeline_element
                && (call.is_call_to_command("complete", context)
                    || call.is_call_to_command("ignore", context))
            {
                continue;
            }
            let violation = create_violation(pipeline, element);
            return vec![violation];
        }
    }

    vec![]
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

struct NonFinalFailureCheck;

impl DetectFix for NonFinalFailureCheck {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "non_final_failure_check"
    }

    fn short_description(&self) -> &'static str {
        "Non-final pipeline command exit code ignored"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/blog/2025-10-15-nushell_v0_108_0.html#pipefail-16449-toc")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(context.ast.detect_in_pipelines(context, check_pipeline))
    }
}

pub static RULE: &dyn Rule = &NonFinalFailureCheck;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
