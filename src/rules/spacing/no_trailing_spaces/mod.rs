use std::sync::OnceLock;

use regex::Regex;

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};
fn trailing_space_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"[ \t]+$").unwrap())
}
fn check(context: &LintContext) -> Vec<Detection> {
    let mut violations = Vec::new();
    let source = unsafe { context.source() };
    let file_offset = context.file_offset();
    let mut byte_offset = 0;

    for (line_num, line) in source.lines().enumerate() {
        if let Some(m) = trailing_space_pattern().find(line) {
            let file_start = byte_offset + m.start();
            let file_end = byte_offset + m.end();
            let violation_span =
                nu_protocol::Span::new(file_start + file_offset, file_end + file_offset);

            violations.push(
                Detection::from_global_span(
                    format!("Line {} has trailing whitespace", line_num + 1),
                    violation_span,
                )
                .with_primary_label("trailing whitespace")
                .with_help("Remove trailing spaces"),
            );
        }
        // Update byte offset for next line (including newline character)
        byte_offset += line.len() + 1;
    }
    violations
}
struct NoTrailingSpaces;

impl DetectFix for NoTrailingSpaces {
    type FixInput = ();

    fn id(&self) -> &'static str {
        "no_trailing_spaces"
    }

    fn explanation(&self) -> &'static str {
        "Eliminate trailing spaces at the end of lines"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/style_guide.html#multi-line-format")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
        Self::no_fix(check(context))
    }
}

pub static RULE: &dyn Rule = &NoTrailingSpaces;
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
