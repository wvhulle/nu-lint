use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;

pub struct PreferParseCommand;

impl PreferParseCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PreferParseCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for PreferParseCommand {
    fn id(&self) -> &str {
        "BP004"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &str {
        "Prefer 'parse' command over manual string splitting with indexed access"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Pattern 1: split row followed by get/skip with index access
        let split_get_pattern =
            Regex::new(r#"split\s+row\s+["'][^"']*["']\s*\|\s*(get\s+\d+|skip\s+\d+)"#).unwrap();

        violations.extend(
            context.violations_from_regex(
                &split_get_pattern,
                self.id(),
                self.severity(),
                "Manual string splitting with indexed access - consider using 'parse'",
                Some(
                    "Use 'parse \"pattern {field1} {field2}\"' for structured text extraction"
                        .to_string(),
                ),
            ),
        );

        // Pattern 2: let parts = ... split row, then $parts | get
        let split_to_var_pattern =
            Regex::new(r#"let\s+(\w+)\s*=\s*\([^)]*split\s+row[^)]*\)"#).unwrap();

        violations.extend(
            split_to_var_pattern
                .find_iter(context.source)
                .filter_map(|mat| {
                    let var_name = mat.as_str().split_whitespace().nth(1)?;
                    // Look for subsequent indexed access
                    let access_pattern =
                        format!(r"\${}?\s*\|\s*(get|skip)\s+\d+", regex::escape(var_name));

                    if Regex::new(&access_pattern).ok()?.is_match(context.source) {
                        Some(Violation {
                            rule_id: self.id().to_string(),
                            severity: self.severity(),
                            message: format!(
                                "Variable '{}' from split row with indexed access - consider using 'parse'",
                                var_name
                            ),
                            span: nu_protocol::Span::new(mat.start(), mat.end()),
                            suggestion: Some(
                                "Use 'parse' command to extract named fields instead of indexed access"
                                    .to_string()
                            ),
                            fix: None,
                            file: None,
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        );

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_row_with_get_detected() {
        let rule = PreferParseCommand::new();

        let bad_code = r#"
let line = "Device AA:BB:CC:DD:EE:FF MyDevice"
let mac = ($line | split row " " | get 1)
"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect split row with indexed get"
        );
    }

    #[test]
    fn test_split_to_variable_with_access() {
        let rule = PreferParseCommand::new();

        let bad_code = r#"
let parts = ($line | split row " ")
let mac = ($parts | get 1)
let name = ($parts | skip 2)
"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect split to variable with indexed access"
        );
    }

    #[test]
    fn test_parse_command_not_flagged() {
        let rule = PreferParseCommand::new();

        let good_code = r#"
let parsed = ($line | parse "Device {mac} {name}")
let mac = $parsed.mac
"#;
        let context = LintContext::test_from_source(good_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag parse command usage"
        );
    }
}
