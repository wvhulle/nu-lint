use std::sync::OnceLock;

use heck::ToShoutySnakeCase;
use regex::Regex;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{RegexRule, RuleCategory, RuleMetadata},
};

#[derive(Default)]
pub struct ScreamingSnakeConstants;

impl ScreamingSnakeConstants {
    fn screaming_snake_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"^[A-Z][A-Z0-9_]*$").unwrap())
    }

    fn const_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"\bconst\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=").unwrap())
    }

    fn is_valid_screaming_snake(name: &str) -> bool {
        Self::screaming_snake_pattern().is_match(name)
    }
}

impl RuleMetadata for ScreamingSnakeConstants {
    fn id(&self) -> &'static str {
        "screaming_snake_constants"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Naming
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "Constants should use SCREAMING_SNAKE_CASE naming convention"
    }
}

impl RegexRule for ScreamingSnakeConstants {
    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let const_pattern = Self::const_pattern();

        const_pattern
            .captures_iter(context.source)
            .filter_map(|cap| {
                let const_match = cap.get(1)?;
                let const_name = const_match.as_str();

                if Self::is_valid_screaming_snake(const_name) {
                    None
                } else {
                    Some(Violation {
                        rule_id: self.id().to_string(),
                        severity: self.severity(),
                        message: format!(
                            "Constant '{const_name}' should use SCREAMING_SNAKE_CASE naming \
                             convention"
                        ),
                        span: nu_protocol::Span::new(const_match.start(), const_match.end()),
                        suggestion: Some(format!(
                            "Consider renaming to: {}",
                            const_name.to_shouty_snake_case()
                        )),
                        fix: None,
                        file: None,
                    })
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
