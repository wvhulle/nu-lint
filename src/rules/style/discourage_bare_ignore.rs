use crate::context::LintContext;
use crate::lint::{Severity, Violation};
use crate::rule::{Rule, RuleCategory};
use regex::Regex;
use std::sync::OnceLock;

pub struct DiscouragedBareIgnore;

impl DiscouragedBareIgnore {
    pub fn new() -> Self {
        Self
    }

    fn ignore_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"\|\s*ignore\s*(?:\n|$)").unwrap())
    }
}

impl Default for DiscouragedBareIgnore {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for DiscouragedBareIgnore {
    fn id(&self) -> &str {
        "S011"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &str {
        "Using '| ignore' may hide errors - consider explicit error handling"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        // Pattern: | ignore (but allow external commands with ^)
        let ignore_pattern = Self::ignore_pattern();

        ignore_pattern
            .find_iter(context.source)
            .filter_map(|mat| {
                // Get some context before the match to see if it's an external command
                let context_start = mat.start().saturating_sub(50);
                let context_str = &context.source[context_start..mat.start()];

                // If it's an external command (^), it's more acceptable
                let is_external = context_str.contains('^');

                if is_external {
                    None
                } else {
                    Some(Violation {
                        rule_id: self.id().to_string(),
                        severity: self.severity(),
                        message: "Piping to 'ignore' suppresses output without error handling"
                            .to_string(),
                        span: nu_protocol::Span::new(mat.start(), mat.end()),
                        suggestion: Some(
                            "Consider: 'do -i { ... }' for error suppression or handle errors explicitly"
                                .to_string()
                        ),
                        fix: None,
                        file: None,
                    })
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bare_ignore_detected() {
        let rule = DiscouragedBareIgnore::new();

        let bad_code = r"
some | pipeline | each { |x| process $x } | ignore
";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect bare ignore"
        );
    }

    #[test]
    fn test_external_command_ignore_acceptable() {
        let rule = DiscouragedBareIgnore::new();

        let acceptable_code = r"
^bluetoothctl power on | ignore
";
        let context = LintContext::test_from_source(acceptable_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag external command ignore"
        );
    }

    #[test]
    fn test_do_ignore_not_flagged() {
        let rule = DiscouragedBareIgnore::new();

        let good_code = r"
do -i { some | pipeline }
";
        let context = LintContext::test_from_source(good_code);
        assert_eq!(rule.check(&context).len(), 0, "Should not flag do -i");
    }
}
