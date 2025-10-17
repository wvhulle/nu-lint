use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{RegexRule, RuleCategory, RuleMetadata},
};

pub struct ExportedFunctionDocs;

impl ExportedFunctionDocs {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExportedFunctionDocs {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleMetadata for ExportedFunctionDocs {
    fn id(&self) -> &'static str {
        "exported_function_docs"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Documentation
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "Exported functions should have documentation comments"
    }
}

impl RegexRule for ExportedFunctionDocs {
    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Search for "export def" patterns in the source code
        let source_lines: Vec<&str> = context.source.lines().collect();

        for (line_idx, line) in source_lines.iter().enumerate() {
            let trimmed = line.trim();

            // Look for export def declarations
            if trimmed.starts_with("export def ") {
                // Check if the line immediately above is a doc comment
                // Doc comments are lines starting with # but not ##, and not test/example
                // comments
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
                        after_def[..bracket_idx].trim()
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
                            "Exported function '{func_name}' is missing documentation"
                        ),
                        span: nu_protocol::Span::new(line_start, line_end),
                        suggestion: Some(format!(
                            "Add a documentation comment above the function:\n# Description of \
                             {func_name}\nexport def {func_name} ..."
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
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
