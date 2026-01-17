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

            Some(
                Detection::from_global_span(
                    format!(
                        "External command '{cmd_name}' can fail silently. Pipe to 'complete' and \
                         check 'exit_code'."
                    ),
                    element.expr.span,
                )
                .with_primary_label("may return non-zero exit code"),
            )
        })
        .collect()
}

struct WrapExternalWithComplete;

impl DetectFix for WrapExternalWithComplete {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "wrap_external_with_complete"
    }

    fn short_description(&self) -> &'static str {
        "External command missing `complete` wrapper"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "External commands can fail and return non-zero exit codes without raising errors. \
             Unlike builtin commands, 'try' blocks do not catch external command failures based \
             on exit code. Use '| complete' to capture stdout, stderr, and exit_code in a record, \
             then check the exit_code field to handle errors properly. Without this, failures may \
             go unnoticed and cause silent data corruption or unexpected behavior downstream.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/complete.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(context.ast.detect_in_pipelines(context, check_pipeline))
    }
}

pub static RULE: &dyn Rule = &WrapExternalWithComplete;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
