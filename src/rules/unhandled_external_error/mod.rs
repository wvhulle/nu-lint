use nu_protocol::ast::{Expr, Pipeline};

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

fn is_piped_to_complete(pipeline: &Pipeline, idx: usize, context: &LintContext) -> bool {
    pipeline.elements.get(idx + 1).is_some_and(|next| {
        matches!(&next.expr.expr, Expr::Call(call) if call.is_call_to_command("complete", context))
    })
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<Detection> {
    pipeline
        .elements
        .iter()
        .enumerate()
        .filter_map(|(i, element)| {
            let Expr::ExternalCall(cmd_expr, args) = &element.expr.expr else {
                return None;
            };

            let cmd_name = cmd_expr.span_text(context);

            if !has_external_side_effect(
                cmd_name,
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                context,
                args,
            ) {
                return None;
            }

            // Skip commands where streaming output is preferred (build tools, etc.)
            // Users often want to see live progress rather than buffering with complete
            if has_external_side_effect(cmd_name, ExternEffect::SlowStreamingOutput, context, args)
            {
                return None;
            }

            if is_piped_to_complete(pipeline, i, context) {
                return None;
            }

            if context
                .ast
                .is_span_inside_try_block(context, element.expr.span)
            {
                return None;
            }

            Some(
                Detection::from_global_span(
                    format!(
                        "External command '{cmd_name}' can fail silently. Wrap in 'try' block or \
                         pipe to 'complete' and check 'exit_code'."
                    ),
                    element.expr.span,
                )
                .with_primary_label("may return non-zero exit code"),
            )
        })
        .collect()
}

struct UnhandledExternalError;

impl DetectFix for UnhandledExternalError {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "unhandled_external_error"
    }

    fn short_description(&self) -> &'static str {
        "Unhandled external command error"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "External commands can fail with non-zero exit codes. Since nushell 0.98.0, these \
             raise errors that 'try' blocks can catch. For cases where you need to inspect \
             stdout, stderr, and exit_code separately, use '| complete' to capture them in a \
             record. Without 'try' or 'complete', failures may go unnoticed and cause silent data \
             corruption or unexpected behavior downstream.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/complete.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(context.ast.detect_in_pipelines(context, check_pipeline))
    }
}

pub static RULE: &dyn Rule = &UnhandledExternalError;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
