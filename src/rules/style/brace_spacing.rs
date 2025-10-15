use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;

pub struct BraceSpacing;

impl BraceSpacing {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BraceSpacing {
    fn default() -> Self {
        Self::new()
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
        // Check for records without proper spacing: {key: value} vs { key: value }
        // Only check for records with colons (key: value patterns)
        let bad_record = Regex::new(r"\{[a-zA-Z_]").unwrap();

        context.violations_from_regex_if(&bad_record, self.id(), self.severity(), |mat| {
            // Look ahead to see if this contains a colon (record pattern)
            let remaining_text = &context.source[mat.start()..];
            if let Some(close_brace_pos) = remaining_text.find('}') {
                let record_content = &remaining_text[..close_brace_pos];

                // Only flag if it contains a colon (key: value pattern)
                if record_content.contains(':') {
                    Some((
                        "Record braces should have spaces: { key: value }".to_string(),
                        Some("Add spaces: { key: value } instead of {key: value}".to_string()),
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bad_brace_spacing() {
        let rule = BraceSpacing::new();

        let bad = "{key: value}";
        let context = LintContext::test_from_source(bad);
        let violations = rule.check(&context);
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_good_brace_spacing() {
        let rule = BraceSpacing::new();

        let good = "{ key: value }";
        let context = LintContext::test_from_source(good);
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    }
}
