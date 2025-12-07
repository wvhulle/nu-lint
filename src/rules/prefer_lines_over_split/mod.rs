use crate::{context::LintContext, rule::Rule, violation::Violation};
fn check(context: &LintContext) -> Vec<Violation> {
    let mut violations = Vec::new();
    // Search for "split row" patterns with newline in the source code
    let source_lines: Vec<&str> = context.source.lines().collect();
    for (line_idx, line) in source_lines.iter().enumerate() {
        // Look for split row with newline patterns
        if line.contains("split row")
            && (line.contains("\"\\n\"")
                || line.contains("'\\n'")
                || line.contains("\"\n\"")
                || line.contains("'\n'"))
        {
            // Calculate the span for this line
            let line_start: usize = source_lines[..line_idx]
                .iter()
                .map(|l| l.len() + 1) // +1 for newline
                .sum();
            let line_end = line_start + line.len();
            let full_span = nu_protocol::Span::new(line_start, line_end);

            // Find the "split row" part
            let split_row_pos = line.find("split row").unwrap_or(0);
            let split_span = nu_protocol::Span::new(
                line_start + split_row_pos,
                line_start + split_row_pos + 9, // "split row" is 9 chars
            );

            violations.push(
                Violation::new(
                    "Use 'lines' instead of 'split row \"\\n\"' for splitting by newlines",
                    full_span,
                )
                .with_primary_label("inefficient newline split")
                .with_extra_label("replace with 'lines'", split_span)
                .with_help(
                    "Replace with: | lines\nThe 'lines' command is more efficient and clearer for \
                     splitting text by newlines.",
                ),
            );
        }
    }
    violations
}
pub const fn rule() -> Rule {
    Rule::new(
        "prefer_lines_over_split",
        "Use 'lines' instead of 'split row \"\\n\"' for better performance and clarity",
        check,
    )
    .with_doc_url("https://www.nushell.sh/commands/docs/lines.html")
}
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
