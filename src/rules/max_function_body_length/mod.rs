use crate::{
    LintLevel,
    ast::declaration,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};
const MAX_LINES: usize = 40;

fn check(context: &LintContext) -> Vec<Detection> {
    context
        .custom_commands()
        .iter()
        .filter_map(|def| function_violation(context, def))
        .collect()
}
fn function_violation(
    context: &LintContext<'_>,
    def: &declaration::CustomCommandDef,
) -> Option<Detection> {
    let block = context.working_set.get_block(def.body);
    let block_span = block.span?;
    let line_count = context.span_text(block_span).lines().count();
    (line_count > MAX_LINES).then(|| {
        let message = format!(
            "Function `{}` has {line_count} lines, which exceeds the maximum of {MAX_LINES} lines",
            def.name
        );
        Detection::from_file_span(message, def.declaration_span(context))
            .with_primary_label(format!("{line_count} lines"))
            .with_extra_label("function body", block_span)
    })
}
struct TooManyLines;

impl DetectFix for TooManyLines {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "max_function_body_length"
    }

    fn short_description(&self) -> &'static str {
        "Function bodies should be short to maintain readability"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Consider refactoring into smaller, more focused functions with clear responsibilities",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/custom_commands.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
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
