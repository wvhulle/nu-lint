use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let mut violations = Vec::new();

    // Search for "error make" patterns in the source code
    let source_lines: Vec<&str> = context.source.lines().collect();

    for (line_idx, line) in source_lines.iter().enumerate() {
        // Look for error make calls
        if line.contains("error make") {
            // Check if the line contains a msg field
            let line_lower = line.to_lowercase();

            // Look for generic/vague error messages
            let has_generic_message = line_lower.contains("msg: \"error\"")
                || line_lower.contains("msg: 'error'")
                || line_lower.contains("msg: \"failed\"")
                || line_lower.contains("msg: 'failed'")
                || line_lower.contains("msg: \"err\"")
                || line_lower.contains("msg: 'err'")
                || line_lower.contains("msg: \"something went wrong\"")
                || line_lower.contains("msg: 'something went wrong'");

            if has_generic_message {
                // Calculate the span for this line
                let line_start: usize = source_lines[..line_idx]
                    .iter()
                    .map(|l| l.len() + 1) // +1 for newline
                    .sum();
                let line_end = line_start + line.len();

                violations.push(
                    RuleViolation::new_static(
                        "descriptive_error_messages",
                        "Error message is too generic and not descriptive",
                        nu_protocol::Span::new(line_start, line_end),
                    )
                    .with_suggestion_static(
                        "Use a descriptive error message that explains what went wrong and how to \
                         fix it.\nExample: error make { msg: \"Failed to parse input: expected \
                         number, got string\" }",
                    ),
                );
            }
        }
    }

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "descriptive_error_messages",
        RuleCategory::ErrorHandling,
        Severity::Warning,
        "Error messages should be descriptive and actionable",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
