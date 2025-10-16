use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};

pub struct PreferLinesOverSplit;

impl PreferLinesOverSplit {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PreferLinesOverSplit {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for PreferLinesOverSplit {
    fn id(&self) -> &str {
        "P002"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Performance
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &str {
        "Use 'lines' instead of 'split row \"\\n\"' for better performance and clarity"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
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
                    rule_id: self.id().to_string(),
                    severity: self.severity(),
                    message: "Use 'lines' instead of 'split row \"\\n\"' for splitting by newlines".to_string(),
                    span: nu_protocol::Span::new(line_start, line_end),
                    suggestion: Some(
                        "Replace with: | lines\n\
                         The 'lines' command is more efficient and clearer for splitting text by newlines."
                            .to_string()
                    ),
                    fix: None,
                    file: None,
                });
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
    fn test_split_row_newline_detected() {
        let source = r#"
def process [] {
    ^git log --oneline | split row "\n"
}
"#;
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None);

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "P002").collect();

        assert!(
            !rule_violations.is_empty(),
            "Should detect split row with newline"
        );
    }

    #[test]
    fn test_lines_not_flagged() {
        let source = r"
def process [] {
    ^git log --oneline | lines
}
";
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None);

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "P002").collect();

        assert!(
            rule_violations.is_empty(),
            "Should not flag 'lines' command"
        );
    }

    #[test]
    fn test_split_row_other_delimiter_not_flagged() {
        let source = r#"
def process [] {
    "a,b,c" | split row ","
}
"#;
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None);

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "P002").collect();

        assert!(
            rule_violations.is_empty(),
            "Should not flag split row with other delimiters"
        );
    }

    #[test]
    fn test_single_quote_newline_detected() {
        let source = r"
def process [] {
    open file.txt | split row '\n'
}
";
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None);

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "P002").collect();

        assert!(
            !rule_violations.is_empty(),
            "Should detect split row with single-quote newline"
        );
    }
}
