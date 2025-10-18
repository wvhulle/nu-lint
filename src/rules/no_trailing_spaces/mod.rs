use std::sync::OnceLock;

use regex::Regex;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{RegexRule, RuleCategory, RuleMetadata},
};

#[derive(Default)]
pub struct NoTrailingSpaces;

impl NoTrailingSpaces {
    fn trailing_space_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"[ \t]+$").unwrap())
    }
}

impl RuleMetadata for NoTrailingSpaces {
    fn id(&self) -> &'static str {
        "no_trailing_spaces"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "Eliminate trailing spaces at the end of lines"
    }
}

impl RegexRule for NoTrailingSpaces {
    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        let source = context.source;
        let lines: Vec<&str> = source.lines().collect();
        let mut byte_offset = 0;

        for (line_num, line) in lines.iter().enumerate() {
            if let Some(m) = Self::trailing_space_pattern().find(line) {
                let violation_start = byte_offset + m.start();
                let violation_end = byte_offset + m.end();

                violations.push(Violation {
                    rule_id: self.id().to_string(),
                    severity: self.severity(),
                    message: format!(
                        "Line {} has trailing whitespace",
                        line_num + 1
                    ),
                    span: nu_protocol::Span::new(violation_start, violation_end),
                    suggestion: Some("Remove trailing spaces".to_string()),
                    fix: None,
                    file: None,
                });
            }

            // Update byte offset for next line (including newline character)
            byte_offset += line.len() + 1;
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