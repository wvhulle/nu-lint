use nu_protocol::ast::Pipeline;

use crate::{
    LintLevel,
    ast::span::SpanExt,
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement, Violation},
};

const MAX_PIPELINE_LENGTH: usize = 100;
const MIN_PIPELINE_ELEMENTS: usize = 3;

fn check(context: &LintContext) -> Vec<Violation> {
    context
        .ast
        .pipelines
        .iter()
        .filter_map(|pipeline| check_pipeline(pipeline, context))
        .collect()
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Option<Violation> {
    if pipeline.elements.len() < MIN_PIPELINE_ELEMENTS {
        return None;
    }

    let span = pipeline_span(pipeline)?;
    let text = span.source_code(context);

    if text.contains('\n') || text.len() <= MAX_PIPELINE_LENGTH {
        return None;
    }

    let fixed = generate_multiline_pipeline(pipeline, context);

    Some(
        Violation::new("Long pipeline should be split across multiple lines", span).with_fix(
            Fix::with_explanation("Format as multiline", vec![Replacement::new(span, fixed)]),
        ),
    )
}

fn generate_multiline_pipeline(pipeline: &Pipeline, context: &LintContext) -> String {
    let mut parts = Vec::new();

    for (i, element) in pipeline.elements.iter().enumerate() {
        let element_text = element.expr.span.source_code(context);
        if i == 0 {
            parts.push(element_text.to_string());
        } else {
            parts.push(format!("| {element_text}"));
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

pub const RULE: Rule = Rule::new(
    "reflow_wide_pipelines",
    "Wrap wide pipelines vertically across multiple lines.",
    check,
    LintLevel::Hint,
)
.with_doc_url("https://www.nushell.sh/book/style_guide.html#multi-line-format");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
