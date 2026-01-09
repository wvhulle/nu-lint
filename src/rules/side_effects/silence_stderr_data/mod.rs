use nu_protocol::ast::{
    Block, Expr, Pipeline, PipelineElement, PipelineRedirection, RedirectionSource,
    RedirectionTarget, Traverse,
};

use crate::{
    LintLevel,
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

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Option<Detection> {
    pipeline
        .elements
        .iter()
        .enumerate()
        .find_map(|(i, element)| {
            let Expr::ExternalCall(head, args) = &element.expr.expr else {
                return None;
            };

            if !is_stderr_silenced(element) || !next_element_is_ignore(pipeline, i, context) {
                return None;
            }

            let cmd_name = context.plain_text(head.span);

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

            let _help_message = format!(
                "Command '{cmd_name}' produces useful output on stderr. Consider removing the \
                 stderr redirection (e>| or o+e>|) to preserve this output, or redirect stderr to \
                 a file for later inspection."
            );

            Some(
                Detection::from_global_span(message, element.expr.span)
                    .with_primary_label("silences stderr data"),
            )
        })
}

fn check_block(block: &Block, context: &LintContext, violations: &mut Vec<Detection>) {
    for pipeline in &block.pipelines {
        violations.extend(check_pipeline(pipeline, context));

        let nested_block_ids: Vec<_> = pipeline
            .elements
            .iter()
            .flat_map(|element| {
                let mut blocks = Vec::new();
                element.expr.flat_map(
                    context.working_set,
                    &|expr| match &expr.expr {
                        Expr::Block(id) | Expr::Closure(id) | Expr::Subexpression(id) => vec![*id],
                        _ => vec![],
                    },
                    &mut blocks,
                );
                blocks
            })
            .collect();

        for block_id in nested_block_ids {
            let nested_block = context.working_set.get_block(block_id);
            check_block(nested_block, context, violations);
        }
    }
}

struct SilenceStderrData;

impl DetectFix for SilenceStderrData {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "silence_stderr_data"
    }

    fn short_description(&self) -> &'static str {
        "External commands that write data to stderr should not have stderr redirected to ignore"
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

pub static RULE: &dyn Rule = &SilenceStderrData;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
