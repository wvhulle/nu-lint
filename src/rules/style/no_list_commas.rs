use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;

pub struct NoListCommas;

impl NoListCommas {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoListCommas {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for NoListCommas {
    fn id(&self) -> &str {
        "S006"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &str {
        "Omit commas between list items (Nushell style guide)"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        // Look for lists with commas like [1, 2, 3]
        let list_comma_pattern = Regex::new(r"\[\s*[^\]]+,\s*[^\]]+\]").unwrap();

        list_comma_pattern
            .find_iter(context.source)
            .filter(|mat| mat.as_str().contains(','))
            .map(|mat| Violation {
                rule_id: self.id().to_string(),
                severity: self.severity(),
                message: "List items should not be separated by commas".to_string(),
                span: nu_protocol::Span::new(mat.start(), mat.end()),
                suggestion: Some(
                    "Remove commas and separate items with spaces: [1 2 3]".to_string(),
                ),
                fix: None,
                file: None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_with_commas() {
        let rule = NoListCommas::new();

        let bad = "[1, 2, 3]";
        let context = LintContext::test_from_source(bad);
        let violations = rule.check(&context);
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_list_without_commas() {
        let rule = NoListCommas::new();

        let good = "[1 2 3]";
        let context = LintContext::test_from_source(good);
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    }
}
