use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;

pub struct SnakeCaseVariables {
    pattern: Regex,
}

impl SnakeCaseVariables {
    pub fn new() -> Self {
        Self {
            pattern: Regex::new(r"^[a-z][a-z0-9_]*$").unwrap(),
        }
    }

    fn is_valid_snake_case(&self, name: &str) -> bool {
        self.pattern.is_match(name)
    }
}

impl Default for SnakeCaseVariables {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for SnakeCaseVariables {
    fn id(&self) -> &str {
        "S001"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &str {
        "Variables should use snake_case naming convention"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let let_pattern = Regex::new(r"\blet\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=").unwrap();

        let_pattern
            .captures_iter(context.source)
            .filter_map(|cap| {
                let var_match = cap.get(1)?;
                let var_name = var_match.as_str();

                if !self.is_valid_snake_case(var_name) {
                    Some(Violation {
                        rule_id: self.id().to_string(),
                        severity: self.severity(),
                        message: format!(
                            "Variable '{}' should use snake_case naming convention",
                            var_name
                        ),
                        span: nu_protocol::Span::new(var_match.start(), var_match.end()),
                        suggestion: Some(format!(
                            "Consider renaming to: {}",
                            to_snake_case(var_name)
                        )),
                        fix: None,
                        file: None,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_lower = false;

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && prev_is_lower {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_is_lower = false;
        } else {
            result.push(c);
            prev_is_lower = c.is_lowercase();
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("myVariable"), "my_variable");
        assert_eq!(to_snake_case("MyVariable"), "my_variable");
        assert_eq!(to_snake_case("my_variable"), "my_variable");
        assert_eq!(to_snake_case("CONSTANT"), "constant");
    }

    #[test]
    fn test_is_valid_snake_case() {
        let rule = SnakeCaseVariables::new();
        assert!(rule.is_valid_snake_case("my_variable"));
        assert!(rule.is_valid_snake_case("x"));
        assert!(rule.is_valid_snake_case("var_2"));
        assert!(!rule.is_valid_snake_case("myVariable"));
        assert!(!rule.is_valid_snake_case("MyVariable"));
        assert!(!rule.is_valid_snake_case("MY_CONSTANT"));
    }
}
