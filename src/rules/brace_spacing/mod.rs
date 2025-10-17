use std::sync::OnceLock;

use regex::Regex;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{RegexRule, RuleCategory, RuleMetadata},
};

#[derive(Default)]
pub struct BraceSpacing;

impl BraceSpacing {
    fn bad_record_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"\{[a-zA-Z_]").unwrap())
    }
}

impl RuleMetadata for BraceSpacing {
    fn id(&self) -> &'static str {
        "brace_spacing"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &'static str {
        "Braces should have one space after opening and before closing"
    }
}

impl RegexRule for BraceSpacing {
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
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
