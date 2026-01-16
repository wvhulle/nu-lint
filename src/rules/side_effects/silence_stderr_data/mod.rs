use nu_protocol::ast::{
    Expr, Pipeline, PipelineElement, PipelineRedirection, RedirectionSource, RedirectionTarget,
};

use crate::{
    LintLevel,
    ast::block::BlockExt,
    context::LintContext,
    effect::external::{ExternEffect, extract_external_arg_text, has_external_side_effect},
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn is_stderr_silenced(element: &PipelineElement) -> bool {
    element
        .redirection
        .as_ref()
        .is_some_and(|redirection| match redirection {
            PipelineRedirection::Single { source, target } => matches!(
                (source, target),
                (
                    RedirectionSource::Stderr | RedirectionSource::StdoutAndStderr,
                    RedirectionTarget::Pipe { .. }
                )
            ),
            PipelineRedirection::Separate { err, .. } => {
                matches!(err, RedirectionTarget::Pipe { .. })
            }
        })
}

fn next_element_is_ignore(
    pipeline: &Pipeline,
    current_index: usize,
    context: &LintContext,
) -> bool {
    pipeline
        .elements
        .get(current_index + 1)
        .is_some_and(|next| match &next.expr.expr {
            Expr::Call(call) => context.working_set.get_decl(call.decl_id).name() == "ignore",
            _ => false,
        })
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<Detection> {
    pipeline
        .elements
        .iter()
        .enumerate()
        .filter_map(|(i, element)| {
            let Expr::ExternalCall(head, args) = &element.expr.expr else {
                return None;
            };

            if !is_stderr_silenced(element) || !next_element_is_ignore(pipeline, i, context) {
                return None;
            }

            let cmd_name = context.span_text(head.span);

            if !has_external_side_effect(cmd_name, ExternEffect::WritesDataToStdErr, context, args)
            {
                log::debug!(
                    "Command '{cmd_name}' does not have WritesDataToStdErr side effect, skipping"
                );
                return None;
            }

            log::debug!(
                "Found stderr silencing for command '{cmd_name}' with WritesDataToStdErr at span \
                 {:?}",
                element.expr.span
            );

            let args_display = args
                .iter()
                .map(|arg| extract_external_arg_text(arg, context))
                .collect::<Vec<_>>()
                .join(" ");

            let args_display = if args_display.is_empty() {
                String::new()
            } else {
                format!(" {args_display}")
            };

            let message = format!(
                "External command '{cmd_name}{args_display}' writes data to stderr but stderr is \
                 redirected to ignore"
            );

            Some(
                Detection::from_global_span(message, element.expr.span)
                    .with_primary_label("silences stderr data"),
            )
        })
        .collect()
}

struct SilenceStderrData;

impl DetectFix for SilenceStderrData {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "silence_stderr_data"
    }

    fn short_description(&self) -> &'static str {
        "External commands that write data to stderr should not be silenced"
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(context.ast.detect_in_pipelines(context, check_pipeline))
    }
}

pub static RULE: &dyn Rule = &SilenceStderrData;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
