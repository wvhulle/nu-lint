use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;

pub struct MissingCommandDocs;

impl MissingCommandDocs {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MissingCommandDocs {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for MissingCommandDocs {
    fn id(&self) -> &str {
        "D001"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Documentation
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &str {
        "Custom commands should have documentation comments"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let def_pattern = Regex::new(r"(?m)^[ \t]*def\s+([a-zA-Z_-][a-zA-Z0-9_-]*)\s*\[").unwrap();

        context.violations_from_regex_if(&def_pattern, self.id(), self.severity(), |mat| {
            let caps = def_pattern.captures(mat.as_str())?;
            let cmd_name = caps.get(1)?.as_str();
            let def_start = mat.start();

            // Check if there's a comment (starting with #) on the line before
            let lines_before: Vec<&str> = context.source[..def_start].lines().collect();
            let has_doc = if let Some(prev_line) = lines_before.last() {
                prev_line.trim().starts_with('#')
            } else {
                false
            };

            if !has_doc {
                Some((
                    format!("Command '{}' is missing documentation comments", cmd_name),
                    Some("Add a comment starting with # above the def statement".to_string()),
                ))
            } else {
                None
            }
        })
    }
}
