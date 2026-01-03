use nu_protocol::{
    Span,
    ast::{Block, Expr, Expression, Pipeline, Traverse},
};

use crate::{
    LintLevel,
    ast::call::CallExt,
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

const MIN_CONSECUTIVE_APPENDS: usize = 2;
const MAX_GAP: usize = 3;

const DATA_PRESERVING_COMMANDS: &[&str] = &["filter", "where", "sort", "unique", "uniq"];

fn is_append_call(expr: &Expression, context: &LintContext) -> bool {
    matches!(&expr.expr, Expr::Call(call) if call.is_call_to_command("append", context))
}

fn is_data_preserving(expr: &Expression, context: &LintContext) -> bool {
    matches!(&expr.expr, Expr::Call(call) 
        if DATA_PRESERVING_COMMANDS.contains(&call.get_call_name(context).as_str()))
}

fn has_valid_gap(pipeline: &Pipeline, start: usize, end: usize, context: &LintContext) -> bool {
    end - start <= MAX_GAP
        && (start + 1..end).all(|i| is_data_preserving(&pipeline.elements[i].expr, context))
}

fn find_append_clusters(pipeline: &Pipeline, context: &LintContext) -> Vec<Vec<usize>> {
    let append_indices: Vec<usize> = pipeline
        .elements
        .iter()
        .enumerate()
        .filter_map(|(idx, elem)| is_append_call(&elem.expr, context).then_some(idx))
        .collect();

    if append_indices.len() < MIN_CONSECUTIVE_APPENDS {
        return Vec::new();
    }

    let mut clusters = Vec::new();
    let mut current = vec![append_indices[0]];

    for window in append_indices.windows(2) {
        let (prev, curr) = (window[0], window[1]);
        if curr - prev == 1 || has_valid_gap(pipeline, prev, curr, context) {
            current.push(curr);
        } else {
            if current.len() >= MIN_CONSECUTIVE_APPENDS {
                clusters.push(current);
            }
            current = vec![curr];
        }
    }

    if current.len() >= MIN_CONSECUTIVE_APPENDS {
        clusters.push(current);
    }

    clusters
}

fn extract_element_spans(pipeline: &Pipeline, cluster: &[usize]) -> Option<Vec<Span>> {
    let first_idx = *cluster.first()?;
    if first_idx == 0 {
        return None;
    }

    let mut spans = vec![pipeline.elements[first_idx - 1].expr.span];

    for &idx in cluster {
        let Expr::Call(call) = &pipeline.elements[idx].expr.expr else {
            return None;
        };
        spans.push(call.get_first_positional_arg()?.span);
    }

    Some(spans)
}

fn create_violation(
    cluster: &[usize],
    element_spans: Vec<Span>,
    pipeline: &Pipeline,
) -> (Detection, FixData) {
    let first_idx = cluster[0];
    let last_idx = *cluster.last().unwrap();

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

    if let Some(first_span) = element_spans.first() {
        detection = detection.with_extra_label("Starting list", *first_span);
    }

    (
        detection,
        FixData {
            replace_span,
            element_spans,
        },
    )
}

fn detect_in_block(block: &Block, context: &LintContext) -> Vec<(Detection, FixData)> {
    let mut violations: Vec<_> = block
        .pipelines
        .iter()
        .flat_map(|pipeline| {
            find_append_clusters(pipeline, context)
                .into_iter()
                .filter_map(|cluster| {
                    let spans = extract_element_spans(pipeline, &cluster)?;
                    Some(create_violation(&cluster, spans, pipeline))
                })
        })
        .collect();

    // Recursively check nested blocks
    for pipeline in &block.pipelines {
        for element in &pipeline.elements {
            element.expr.flat_map(
                context.working_set,
                &|expr| match &expr.expr {
                    Expr::Block(id) | Expr::Closure(id) | Expr::Subexpression(id) => {
                        detect_in_block(context.working_set.get_block(*id), context)
                    }
                    _ => vec![],
                },
                &mut violations,
            );
        }
    }

    violations
}

impl DetectFix for ChainedAppend {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "chained_append"
    }

    fn explanation(&self) -> &'static str {
        "Use spread syntax instead of chained 'append' commands"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/operators.html#spread-operator")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        detect_in_block(context.ast, context)
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let element_spans: &[Span] = &fix_data.element_spans;
        let context: &LintContext = context;
        let elements: Vec<String> = element_spans
            .iter()
            .map(|span| format!("...{}", context.plain_text(*span).trim()))
            .collect();

        let fix = format!("[{}]", elements.join(", "));

        Some(Fix::with_explanation(
            "Replace chained appends with spread syntax",
            vec![Replacement::new(fix_data.replace_span, fix)],
        ))
    }
}
