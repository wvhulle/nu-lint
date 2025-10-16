use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};

pub struct DescriptiveErrorMessages;

impl DescriptiveErrorMessages {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for DescriptiveErrorMessages {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for DescriptiveErrorMessages {
    fn id(&self) -> &'static str {
        "BP011"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "Error messages should be descriptive and actionable"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
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

                    violations.push(Violation {
                        rule_id: self.id().to_string(),
                        severity: self.severity(),
                        message: "Error message is too generic and not descriptive".to_string(),
                        span: nu_protocol::Span::new(line_start, line_end),
                        suggestion: Some(
                            "Use a descriptive error message that explains what went wrong and how to fix it.\n\
                             Example: error make { msg: \"Failed to parse input: expected number, got string\" }"
                                .to_string()
                        ),
                        fix: None,
                        file: None,
                    });
                }
            }
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::engine::LintEngine;

    #[test]
    fn test_generic_error_message_detected() {
        let source = r#"
def process [] {
    if $condition {
        error make { msg: "error" }
    }
}
"#;
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None).unwrap();

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "BP011").collect();

        assert!(
            !rule_violations.is_empty(),
            "Should detect generic error message"
        );
    }

    #[test]
    fn test_descriptive_error_message_not_flagged() {
        let source = r#"
def process [input: int] {
    if $input < 0 {
        error make { msg: "Input must be non-negative, got: " + ($input | into string) }
    }
}
"#;
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None).unwrap();

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "BP011").collect();

        assert!(
            rule_violations.is_empty(),
            "Should not flag descriptive error messages"
        );
    }

    #[test]
    fn test_failed_error_message_detected() {
        let source = r#"
def process [] {
    error make { msg: "failed" }
}
"#;
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None).unwrap();

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "BP011").collect();

        assert!(
            !rule_violations.is_empty(),
            "Should detect 'failed' as generic message"
        );
    }
}
