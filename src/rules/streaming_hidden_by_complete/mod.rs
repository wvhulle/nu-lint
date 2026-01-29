use nu_protocol::{
    Span,
    ast::{Expr, Pipeline},
};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt, expression::ExpressionExt},
    context::LintContext,
    effect::external::{ExternEffect, has_external_side_effect},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

/// Data needed to generate a fix for redundant complete on streaming commands
pub struct FixData {
    /// Span of the entire pipeline element including `| complete`
    full_span: Span,
    /// The external command text to preserve
    external_text: String,
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<(Detection, FixData)> {
    pipeline
        .elements
        .windows(2)
        .filter_map(|window| {
            let (first, second) = (&window[0], &window[1]);

            // First element must be an external call with streaming output
            let Expr::ExternalCall(cmd_expr, args) = &first.expr.expr else {
                return None;
            };
            let cmd_name = cmd_expr.span_text(context);
            if !has_external_side_effect(cmd_name, ExternEffect::SlowStreamingOutput, context, args)
            {
                return None;
            }

            // Second element must be `complete`
            let Expr::Call(call) = &second.expr.expr else {
                return None;
            };
            if !call.is_call_to_command("complete", context) {
                return None;
            }

            let detection = Detection::from_global_span(
                format!(
                    "Streaming command '{cmd_name}' wrapped with 'complete' buffers all output. \
                     Remove 'complete' to see live progress."
                ),
                second.expr.span,
            )
            .with_primary_label("buffers streaming output")
            .with_extra_label("produces streaming output", first.expr.span);

            let fix_data = FixData {
                full_span: Span::new(first.expr.span.start, second.expr.span.end),
                external_text: context.expr_text(&first.expr).to_owned(),
            };

            Some((detection, fix_data))
        })
        .collect()
}

struct RedundantCompleteStreaming;

impl DetectFix for RedundantCompleteStreaming {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "streaming_hidden_by_complete"
    }

    fn short_description(&self) -> &'static str {
        "Streaming commands should not be wrapped with 'complete'"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Commands like 'cargo build', 'docker pull', 'git clone', etc. produce streaming \
             output (progress bars, build logs) that is useful to see in real-time. Wrapping \
             these with '| complete' buffers all output until the command finishes, hiding \
             progress information. Remove 'complete' to see live output.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/complete.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Off
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.ast.detect_in_pipelines(context, check_pipeline)
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix {
            explanation: "Remove 'complete' to see streaming output".into(),
            replacements: vec![Replacement::new(
                fix_data.full_span,
                fix_data.external_text.clone(),
            )],
        })
    }
}

pub static RULE: &dyn Rule = &RedundantCompleteStreaming;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
