use nu_protocol::{Id, marker::Block};

use crate::{context::LintContext, rule::Rule, violation::Violation};
const MAX_LINES: usize = 40;

fn check(context: &LintContext) -> Vec<Violation> {
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
) -> Option<Violation> {
    let block = context.working_set.get_block(block_id);
    let function_span = context.find_declaration_span(function_name);
    let block_span = block.span?;
    let line_count = context.get_span_text(block_span).lines().count();
    (line_count > MAX_LINES).then(|| {
        let message = format!(
            "Function `{function_name}` has {line_count} lines, which exceeds the maximum of \
             {MAX_LINES} lines"
        );
        let suggestion = format!(
            "Consider refactoring `{function_name}` into smaller, more focused functions. Break \
             down complex logic into helper functions with clear responsibilities."
        );
        Violation::new(message, function_span)
            .with_primary_label(format!("{line_count} lines"))
            .with_help(suggestion)
    })
}
pub const fn rule() -> Rule {
    Rule::new(
        "max_function_body_length",
        "Function bodies should not exceed 80 lines to maintain readability",
        check,
    )
    .with_doc_url("https://www.nushell.sh/book/custom_commands.html")
}
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
