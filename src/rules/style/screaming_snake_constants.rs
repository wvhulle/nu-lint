use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;

pub struct ScreamingSnakeConstants {
    pattern: Regex,
}

impl ScreamingSnakeConstants {
    pub fn new() -> Self {
        Self {
            pattern: Regex::new(r"^[A-Z][A-Z0-9_]*$").unwrap(),
        }
    }

    fn is_valid_screaming_snake(&self, name: &str) -> bool {
        self.pattern.is_match(name)
    }
}

impl Default for ScreamingSnakeConstants {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for ScreamingSnakeConstants {
    fn id(&self) -> &str {
        "S003"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &str {
        "Constants should use SCREAMING_SNAKE_CASE naming convention"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let const_pattern = Regex::new(r"\bconst\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=").unwrap();

        const_pattern
            .captures_iter(context.source)
            .filter_map(|cap| {
                let const_match = cap.get(1)?;
                let const_name = const_match.as_str();

                if !self.is_valid_screaming_snake(const_name) {
                    Some(Violation {
                        rule_id: self.id().to_string(),
                        severity: self.severity(),
                        message: format!(
                            "Constant '{}' should use SCREAMING_SNAKE_CASE naming convention",
                            const_name
                        ),
                        span: nu_protocol::Span::new(const_match.start(), const_match.end()),
                        suggestion: Some(format!(
                            "Consider renaming to: {}",
                            to_screaming_snake(const_name)
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

fn to_screaming_snake(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_lower = false;

    for (i, c) in s.chars().enumerate() {
        if c == '-' {
            result.push('_');
            prev_is_lower = false;
        } else if c.is_uppercase() {
            if i > 0 && prev_is_lower {
                result.push('_');
            }
            result.push(c);
            prev_is_lower = false;
        } else if c.is_lowercase() {
            result.push(c.to_uppercase().next().unwrap());
            prev_is_lower = true;
        } else {
            result.push(c);
            prev_is_lower = false;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_screaming_snake() {
        assert_eq!(to_screaming_snake("maxValue"), "MAX_VALUE");
        assert_eq!(to_screaming_snake("max_value"), "MAX_VALUE");
        assert_eq!(to_screaming_snake("MAX_VALUE"), "MAX_VALUE");
    }

    #[test]
    fn test_is_valid_screaming_snake() {
        let rule = ScreamingSnakeConstants::new();
        assert!(rule.is_valid_screaming_snake("MAX_VALUE"));
        assert!(rule.is_valid_screaming_snake("CONSTANT"));
        assert!(rule.is_valid_screaming_snake("MY_CONSTANT_2"));
        assert!(!rule.is_valid_screaming_snake("maxValue"));
        assert!(!rule.is_valid_screaming_snake("max_value"));
    }
}
