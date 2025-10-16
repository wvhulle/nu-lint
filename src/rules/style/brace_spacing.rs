use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;
use std::sync::OnceLock;

#[derive(Default)]
pub struct BraceSpacing;

impl BraceSpacing {
    fn bad_record_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"\{[a-zA-Z_]").unwrap())
    }
}

impl Rule for BraceSpacing {
    fn id(&self) -> &str {
        "S007"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &str {
        "Braces should have one space after opening and before closing"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        context.violations_from_regex_if(
            Self::bad_record_pattern(),
            self.id(),
            self.severity(),
            |mat| {
                // Only flag if this is a record (contains ':' before closing '}')
                context.source[mat.start()..]
                    .find('}')
                    .and_then(|close_pos| {
                        context.source[mat.start()..mat.start() + close_pos]
                            .contains(':')
                            .then_some((
                                "Record braces should have spaces: { key: value }".to_string(),
                                Some(
                                    "Add spaces: { key: value } instead of {key: value}"
                                        .to_string(),
                                ),
                            ))
                    })
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bad_brace_spacing() {
        let rule = BraceSpacing::default();

        let bad = "{key: value}";
        let context = LintContext::test_from_source(bad);
        let violations = rule.check(&context);
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_good_brace_spacing() {
        let rule = BraceSpacing::default();

        let good = "{ key: value }";
        let context = LintContext::test_from_source(good);
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    }
}
