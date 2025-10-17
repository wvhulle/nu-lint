use crate::context::LintContext;
use crate::lint::{Severity, Violation};
use crate::rule::{Rule, RuleCategory};
use regex::Regex;

pub struct PreferParseCommand;

impl PreferParseCommand {
    #[must_use]
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
    fn id(&self) -> &'static str {
        "prefer_parse_command"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "Prefer 'parse' command over manual string splitting with indexed access"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Pattern 1: split row followed by get/skip with index access
        let split_get_pattern =
            Regex::new(r#"split\s+row\s+["'][^"']*["']\s*\|\s*(get\s+\d+|skip\s+\d+)"#).unwrap();

        violations.extend(context.violations_from_regex(
            &split_get_pattern,
            self.id(),
            self.severity(),
            "Manual string splitting with indexed access - consider using 'parse'",
            Some("Use 'parse \"pattern {field1} {field2}\"' for structured text extraction"),
        ));

        // Pattern 2: let parts = ... split row, then $parts | get
        let split_to_var_pattern =
            Regex::new(r"let\s+(\w+)\s*=\s*\([^)]*split\s+row[^)]*\)").unwrap();

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
                                "Variable '{var_name}' from split row with indexed access - consider using 'parse'"
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
mod detect_bad;
#[cfg(test)]
mod ignore_good;
#[cfg(test)]
mod generated_fix;