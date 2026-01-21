use nu_protocol::{
    Span,
    ast::{Expr, Pipeline},
};

use crate::{
    Fix, LintLevel, Replacement,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct FixData {
    /// Spans of `$in | ` patterns to remove (including the `$in`, space, and
    /// `|` with trailing space)
    redundant_in_spans: Vec<Span>,
}

/// Find the span of `$in | ` at the start of a pipeline (if present)
/// Returns the span from `$in` start to just after `| ` (including trailing
/// whitespace)
fn find_redundant_in_span(pipeline: &Pipeline, context: &LintContext) -> Option<Span> {
    let [element] = pipeline.elements.as_slice() else {
        return None;
    };

    let Expr::Collect(_, inner_expr) = &element.expr.expr else {
        return None;
    };

    let Expr::Subexpression(block_id) = &inner_expr.expr else {
        return None;
    };

    let inner_block = context.working_set.get_block(*block_id);
    let [inner_pipeline] = inner_block.pipelines.as_slice() else {
        return None;
    };

    // Need at least 2 elements: `$in | command`
    if inner_pipeline.elements.len() < 2 {
        return None;
    }

    let first = inner_pipeline.elements.first()?;

    // Check if first element is $in variable
    if !matches!(&first.expr.expr, Expr::Var(_) | Expr::FullCellPath(_)) {
        return None;
    }

    // Get the span from $in to just before the second element
    let second = inner_pipeline.elements.get(1)?;
    let in_span_start = first.expr.span.start;
    let second_start = second.expr.span.start;

    // Return span that covers `$in | ` (from $in to just before the next command)
    Some(Span::new(in_span_start, second_start))
}

/// Collect all redundant `$in | ` spans from a block's pipelines
fn collect_redundant_in_spans(pipelines: &[Pipeline], context: &LintContext) -> Vec<Span> {
    pipelines
        .iter()
        .filter_map(|p| find_redundant_in_span(p, context))
        .collect()
}

struct RemoveRedundantIn;

impl DetectFix for RemoveRedundantIn {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "remove_redundant_in"
    }

    fn short_description(&self) -> &'static str {
        "Redundant `$in` at pipeline start"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/special_variables.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .custom_commands()
            .iter()
            .filter_map(|def| {
                let signature = &def.signature;
                let block = context.working_set.get_block(def.body);

                let redundant_in_spans = collect_redundant_in_spans(&block.pipelines, context);

                if redundant_in_spans.is_empty() {
                    return None;
                }

                let mut violation = Detection::from_file_span(
                    format!("Redundant $in in function '{}'", signature.name),
                    def.declaration_span(context),
                )
                .with_primary_label("function with redundant $in");

                if let Some(body_span) = block.span {
                    violation = violation.with_extra_label("$in used at pipeline start", body_span);
                }

                let fix_data = FixData { redundant_in_spans };

                Some((violation, fix_data))
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        if fix_data.redundant_in_spans.is_empty() {
            return None;
        }

        // Create replacements that remove each `$in | ` pattern
        let replacements: Vec<Replacement> = fix_data
            .redundant_in_spans
            .iter()
            .map(|span| Replacement::new(*span, String::new()))
            .collect();

        Some(Fix {
            explanation: "Remove redundant $in at pipeline start".into(),
            replacements,
        })
    }
}

pub static RULE: &dyn Rule = &RemoveRedundantIn;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
