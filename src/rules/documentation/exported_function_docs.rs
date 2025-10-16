use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};

pub struct ExportedFunctionDocs;

impl ExportedFunctionDocs {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExportedFunctionDocs {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for ExportedFunctionDocs {
    fn id(&self) -> &str {
        "D002"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Documentation
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &str {
        "Exported functions should have documentation comments"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Search for "export def" patterns in the source code
        let source_lines: Vec<&str> = context.source.lines().collect();

        for (line_idx, line) in source_lines.iter().enumerate() {
            let trimmed = line.trim();

            // Look for export def declarations
            if trimmed.starts_with("export def ") {
                // Check if the line immediately above is a doc comment
                // Doc comments are lines starting with # but not ##, and not test/example comments
                let has_doc_comment = if line_idx > 0 {
                    let prev_line = source_lines[line_idx - 1].trim();
                    let is_comment = prev_line.starts_with('#') && !prev_line.starts_with("##");

                    // Exclude common non-documentation comment patterns
                    let is_test_comment = prev_line.to_lowercase().contains("bad:")
                        || prev_line.to_lowercase().contains("good:")
                        || prev_line.to_lowercase().contains("todo:")
                        || prev_line.to_lowercase().contains("fixme:")
                        || prev_line.to_lowercase().contains("test:")
                        || prev_line.to_lowercase().contains("example:");

                    is_comment && !is_test_comment
                } else {
                    false
                };

                if !has_doc_comment {
                    // Extract function name
                    let after_def = trimmed.strip_prefix("export def ").unwrap();
                    let func_name = if let Some(space_idx) = after_def.find(' ') {
                        &after_def[..space_idx]
                    } else if let Some(bracket_idx) = after_def.find('[') {
                        &after_def[..bracket_idx].trim()
                    } else {
                        after_def
                    };

                    // Calculate the span for this line
                    let line_start: usize = source_lines[..line_idx]
                        .iter()
                        .map(|l| l.len() + 1) // +1 for newline
                        .sum();
                    let line_end = line_start + line.len();

                    violations.push(Violation {
                        rule_id: self.id().to_string(),
                        severity: self.severity(),
                        message: format!(
                            "Exported function '{}' is missing documentation",
                            func_name
                        ),
                        span: nu_protocol::Span::new(line_start, line_end),
                        suggestion: Some(format!(
                            "Add a documentation comment above the function:\n# Description of {}\nexport def {} ...",
                            func_name,
                            func_name
                        )),
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
    use super::*;
    use crate::config::Config;
    use crate::engine::LintEngine;

    #[test]
    fn test_exported_function_without_docs() {
        let source = r#"
export def my-command [] {
    echo "hello"
}
"#;
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None).unwrap();

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "D002").collect();

        assert!(
            !rule_violations.is_empty(),
            "Should detect exported function without docs"
        );
    }

    #[test]
    fn test_exported_function_with_docs() {
        let source = r#"
# This is a documented command
export def my-command [] {
    echo "hello"
}
"#;
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None).unwrap();

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "D002").collect();

        assert!(
            rule_violations.is_empty(),
            "Should not flag documented exported functions"
        );
    }

    #[test]
    fn test_non_exported_function_without_docs() {
        let source = r#"
def my-command [] {
    echo "hello"
}
"#;
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None).unwrap();

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "D002").collect();

        assert!(
            rule_violations.is_empty(),
            "Should not flag non-exported functions"
        );
    }
}
