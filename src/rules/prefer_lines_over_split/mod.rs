use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
};

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

            violations.push(Violation {
                rule_id: "prefer_lines_over_split".to_string().into(),
                severity: Severity::Info,
                message: "Use 'lines' instead of 'split row \"\\n\"' for splitting by newlines"
                    .to_string()
                    .into(),
                span: nu_protocol::Span::new(line_start, line_end),
                suggestion: Some(
                    "Replace with: | lines\nThe 'lines' command is more efficient and clearer for \
                     splitting text by newlines."
                        .to_string()
                        .into(),
                ),
                fix: None,
                file: None,
            });
        }
    }

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_lines_over_split",
        RuleCategory::Idioms,
        Severity::Info,
        "Use 'lines' instead of 'split row \"\\n\"' for better performance and clarity",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
