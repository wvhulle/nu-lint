use nu_protocol::{
    Span,
    ast::{Expr, Pipeline},
};

use crate::{
    LintLevel,
    ast::{
        block::BlockExt,
        call::CallExt,
        pipeline::{ClusterConfig, PipelineExt},
    },
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;

pub struct ChainedAppend;

pub static RULE: &dyn Rule = &ChainedAppend;

pub struct FixData {
    replace_span: Span,
    element_spans: Vec<Span>,
}

const DATA_PRESERVING_COMMANDS: &[&str] = &["filter", "where", "sort", "unique", "uniq"];

fn extract_element_spans(pipeline: &Pipeline, indices: &[usize]) -> Option<Vec<Span>> {
    let first_idx = *indices.first()?;
    if first_idx == 0 {
        return None;
    }

    let mut spans = vec![pipeline.elements[first_idx - 1].expr.span];

    for &idx in indices {
        let Expr::Call(call) = &pipeline.elements[idx].expr.expr else {
            return None;
        };
        spans.push(call.get_first_positional_arg()?.span);
    }

    Some(spans)
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<(Detection, FixData)> {
    let config = ClusterConfig::min_consecutive(2)
        .with_max_gap(3)
        .with_allowed_gaps(DATA_PRESERVING_COMMANDS.to_vec());

    pipeline
        .find_command_clusters("append", context, &config)
        .into_iter()
        .filter_map(|cluster| {
            let spans = extract_element_spans(pipeline, &cluster.indices)?;
            let first_idx = cluster.first_index()?;
            let last_idx = cluster.last_index()?;

            let replace_span = Span::new(
                pipeline.elements[first_idx - 1].expr.span.start,
                pipeline.elements[last_idx].expr.span.end,
            );

            let message = format!(
                "Consider using spread syntax instead of {} chained 'append' operations",
                cluster.len()
            );

            let mut detection = Detection::from_global_span(message, replace_span)
                .with_extra_label("First append", pipeline.elements[first_idx].expr.span)
                .with_extra_label("Last append", pipeline.elements[last_idx].expr.span);

            if let Some(first_span) = spans.first() {
                detection = detection.with_extra_label("Starting list", *first_span);
            }

            Some((
                detection,
                FixData {
                    replace_span,
                    element_spans: spans,
                },
            ))
        })
        .collect()
}

impl DetectFix for ChainedAppend {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "chained_append"
    }

    fn short_description(&self) -> &'static str {
        "Use spread syntax instead of chained 'append' commands"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/operators.html#spread-operator")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.ast.detect_in_pipelines(context, check_pipeline)
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let element_spans: &[Span] = &fix_data.element_spans;
        let context: &LintContext = context;
        let elements: Vec<String> = element_spans
            .iter()
            .map(|span| format!("...{}", context.span_text(*span).trim()))
            .collect();

        let fix = format!("[{}]", elements.join(", "));

        Some(Fix {
            explanation: "Replace chained appends with spread syntax".into(),
            replacements: vec![Replacement::new(fix_data.replace_span, fix)],
        })
    }
}
