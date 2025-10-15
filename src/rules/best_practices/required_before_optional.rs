use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};

pub struct RequiredBeforeOptional;

impl RequiredBeforeOptional {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RequiredBeforeOptional {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for RequiredBeforeOptional {
    fn id(&self) -> &str {
        "BP006"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn description(&self) -> &str {
        "Required parameters must appear before optional parameters"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        context
            .new_user_functions()
            .filter_map(|(_, decl)| {
                let signature = decl.signature();

                // Check if there's a required parameter after an optional one
                // We need to check the positional parameters in order
                let mut seen_optional = false;
                
                // Check required positionals (these should all come first)
                for _ in &signature.required_positional {
                    if seen_optional {
                        // Found a required param after optional - this is an error
                        return Some(Violation {
                            rule_id: self.id().to_string(),
                            severity: self.severity(),
                            message: format!(
                                "Function '{}' has required parameter after optional parameter",
                                signature.name
                            ),
                            span: context.find_declaration_span(&signature.name),
                            suggestion: Some(
                                "Move required parameters before optional parameters in function signature"
                                    .to_string()
                            ),
                            fix: None,
                            file: None,
                        });
                    }
                }
                
                // Mark that we've seen optional positionals
                if !signature.optional_positional.is_empty() {
                    seen_optional = true;
                }
                
                None
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_after_optional_detected() {
        let rule = RequiredBeforeOptional::new();

        let bad_code = r#"
def bad-func [optional?: string, required: string] {
    echo "wrong order"
}
"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect required param after optional"
        );
    }

    #[test]
    fn test_correct_order_not_flagged() {
        let rule = RequiredBeforeOptional::new();

        let good_code = r#"
def good-func [required: string, optional?: string] {
    echo "correct order"
}
"#;
        let context = LintContext::test_from_source(good_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag correct parameter order"
        );
    }

    #[test]
    fn test_only_optional_params_not_flagged() {
        let rule = RequiredBeforeOptional::new();

        let good_code = r#"
def optional-func [opt1?: string, opt2?: int] {
    echo "all optional"
}
"#;
        let context = LintContext::test_from_source(good_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag all-optional params"
        );
    }

    #[test]
    fn test_flags_not_affected() {
        let rule = RequiredBeforeOptional::new();

        let good_code = r#"
def with-flags [optional?: string, --flag: bool] {
    echo "flags can be anywhere"
}
"#;
        let context = LintContext::test_from_source(good_code);
        assert_eq!(rule.check(&context).len(), 0, "Should not flag flags");
    }
}
