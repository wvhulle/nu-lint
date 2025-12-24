use std::sync::OnceLock;

use regex::Regex;

use crate::{LintLevel, context::LintContext, rule::Rule, violation::Violation};
fn trailing_space_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"[ \t]+$").unwrap())
}
fn check(context: &LintContext) -> Vec<Violation> {
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
                Violation::new(
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
pub const RULE: Rule = Rule::new(
    "no_trailing_spaces",
    "Eliminate trailing spaces at the end of lines",
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
