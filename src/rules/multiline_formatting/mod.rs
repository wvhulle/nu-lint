use std::sync::OnceLock;

use regex::Regex;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{RegexRule, RuleCategory, RuleMetadata},
};

#[derive(Default)]
pub struct MultilineFormatting;

impl MultilineFormatting {
    fn complex_construct_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| {
            Regex::new(r"(?m)^\s*(?:def|export def|if|match|for|while|try)\s+[^{]*\{[^}]*\}.*$")
                .unwrap()
        })
    }

    fn multiline_list_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"\[[^\[\]]*[,\s][^\[\]]*\]").unwrap())
    }

    fn multiline_record_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"\{[^{}]*:[^{}]*[,\s][^{}]*\}").unwrap())
    }
}

impl RuleMetadata for MultilineFormatting {
    fn id(&self) -> &'static str {
        "multiline_formatting"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Formatting
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &'static str {
        "Prefer multi-line format for complex constructs and longer lists/records"
    }
}

impl RegexRule for MultilineFormatting {
    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        let source = context.source;
        let lines: Vec<&str> = source.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            // Check for complex constructs that should be multi-line
            if Self::complex_construct_pattern().is_match(line) && line.len() > 80 {
                let line_start = source[..]
                    .lines()
                    .take(line_num)
                    .map(|l| l.len() + 1)
                    .sum::<usize>();
                violations.push(Violation {
                    rule_id: self.id().to_string(),
                    severity: self.severity(),
                    message: "Complex constructs should use multi-line format for better \
                              readability"
                        .to_string(),
                    span: nu_protocol::Span::new(line_start, line_start + line.len()),
                    suggestion: Some(
                        "Consider breaking this construct across multiple lines".to_string(),
                    ),
                    fix: None,
                    file: None,
                });
            }

            // Check for lists that should be multi-line
            if Self::multiline_list_pattern().is_match(line) && line.len() > 60 {
                let line_start = source[..]
                    .lines()
                    .take(line_num)
                    .map(|l| l.len() + 1)
                    .sum::<usize>();
                violations.push(Violation {
                    rule_id: self.id().to_string(),
                    severity: self.severity(),
                    message: "Long lists should use multi-line format with each item on a \
                              separate line"
                        .to_string(),
                    span: nu_protocol::Span::new(line_start, line_start + line.len()),
                    suggestion: Some("Put each list item on a separate line".to_string()),
                    fix: None,
                    file: None,
                });
            }

            // Check for records that should be multi-line
            if Self::multiline_record_pattern().is_match(line) && line.len() > 60 {
                let line_start = source[..]
                    .lines()
                    .take(line_num)
                    .map(|l| l.len() + 1)
                    .sum::<usize>();
                violations.push(Violation {
                    rule_id: self.id().to_string(),
                    severity: self.severity(),
                    message: "Long records should use multi-line format with each field on a \
                              separate line"
                        .to_string(),
                    span: nu_protocol::Span::new(line_start, line_start + line.len()),
                    suggestion: Some("Put each record field on a separate line".to_string()),
                    fix: None,
                    file: None,
                });
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
