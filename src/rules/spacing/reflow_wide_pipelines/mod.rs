use nu_protocol::{Span, ast::Pipeline};

use crate::{
    LintLevel,
    config::PipelinePlacement,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const MIN_PIPELINE_ELEMENTS: usize = 3;

/// Semantic fix data: stores the pipeline span and element spans for
/// reformatting
pub struct FixData {
    pipeline_span: Span,
    element_spans: Vec<Span>,
}

fn detect_pipeline(pipeline: &Pipeline, context: &LintContext) -> Option<(Detection, FixData)> {
    if pipeline.elements.len() < MIN_PIPELINE_ELEMENTS {
        return None;
    }

    let span = pipeline_span(pipeline)?;
    let text = context.plain_text(span);
    let max_length = context.config.max_pipeline_length;

    if text.contains('\n') || text.len() <= max_length {
        return None;
    }

    let element_spans: Vec<Span> = pipeline.elements.iter().map(|e| e.expr.span).collect();
    let violation = Detection::from_global_span(
        format!(
            "Pipeline of {} characters exceeds {} character limit and should be split across \
             multiple lines",
            text.len(),
            max_length
        ),
        span,
    );
    let fix_data = FixData {
        pipeline_span: span,
        element_spans,
    };

    Some((violation, fix_data))
}

fn generate_multiline_pipeline(
    element_spans: &[Span],
    context: &LintContext,
    placement: PipelinePlacement,
) -> String {
    let mut parts = Vec::new();

    match placement {
        PipelinePlacement::Start => {
            for (i, span) in element_spans.iter().enumerate() {
                let element_text = context.plain_text(*span);
                if i == 0 {
                    parts.push(element_text.to_string());
                } else {
                    parts.push(format!("| {element_text}"));
                }
            }
        }
        PipelinePlacement::End => {
            for (i, span) in element_spans.iter().enumerate() {
                let element_text = context.plain_text(*span);
                if i == element_spans.len() - 1 {
                    parts.push(element_text.to_string());
                } else {
                    parts.push(format!("{element_text} |"));
                }
            }
        }
    }

    parts.join("\n")
}

fn pipeline_span(pipeline: &Pipeline) -> Option<nu_protocol::Span> {
    let first = pipeline.elements.first()?;
    let last = pipeline.elements.last()?;
    Some(nu_protocol::Span::new(
        first.expr.span.start,
        last.expr.span.end,
    ))
}

struct ReflowWidePipelines;

impl DetectFix for ReflowWidePipelines {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "reflow_wide_pipelines"
    }

    fn short_description(&self) -> &'static str {
        "Pipelines exceeding the configured character limit should be split across multiple lines \
         for readability"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/style_guide.html#multi-line-format")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .ast
            .pipelines
            .iter()
            .filter_map(|pipeline| detect_pipeline(pipeline, context))
            .collect()
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let placement = context.config.pipeline_placement;
        let fixed = generate_multiline_pipeline(&fix_data.element_spans, context, placement);
        Some(Fix::with_explanation(
            "Format as multiline",
            vec![Replacement::new(fix_data.pipeline_span, fixed)],
        ))
    }
}

pub static RULE: &dyn Rule = &ReflowWidePipelines;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
