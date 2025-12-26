use nu_protocol::{Id, marker::Block};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};
const MAX_LINES: usize = 40;

fn check(context: &LintContext) -> Vec<Detection> {
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
) -> Option<Detection> {
    let block = context.working_set.get_block(block_id);
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
        Detection::from_global_span(message, block_span)
            .with_primary_label(format!("{line_count} lines"))
            .with_help(suggestion)
    })
}
struct TooManyLines;

impl DetectFix for TooManyLines {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "too_many_lines"
    }

    fn explanation(&self) -> &'static str {
        "Function bodies should be short to maintain readability"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/custom_commands.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(check(context))
    }
}

pub static RULE: &dyn Rule = &TooManyLines;
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
