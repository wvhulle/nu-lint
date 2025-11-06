use nu_protocol::{Id, marker::Block};

use crate::{
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

const MAX_LINES: usize = 80;

fn count_lines_in_span(source: &str, span: nu_protocol::Span) -> usize {
    let start = span.start;
    let end = span.end;

    if start >= source.len() || end > source.len() || start >= end {
        return 0;
    }

    source[start..end].lines().count()
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context
        .collect_function_definitions()
        .iter()
        .filter_map(|(block_id, function_name)| {
            function_violation(context, *block_id, function_name)
        })
        .collect()
}

fn function_violation(
    context: &LintContext<'_>,
    block_id: Id<Block>,
    function_name: &String,
) -> Option<RuleViolation> {
    let block = context.working_set.get_block(block_id);
    let function_span = context.find_declaration_span(function_name);

    let block_span = block.span?;
    let line_count = count_lines_in_span(context.source, block_span);

    (line_count > MAX_LINES).then(|| {
        let message = format!(
            "Function `{function_name}` has {line_count} lines, which exceeds the maximum of \
             {MAX_LINES} lines"
        );

        let suggestion = format!(
            "Consider refactoring `{function_name}` into smaller, more focused functions. Break \
             down complex logic into helper functions with clear responsibilities."
        );

        RuleViolation::new_dynamic("max_function_body_length", message, function_span)
            .with_suggestion_dynamic(suggestion)
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "max_function_body_length",
        RuleCategory::CodeQuality,
        Severity::Warning,
        "Function bodies should not exceed 80 lines to maintain readability",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
