use crate::context::LintContext;
use crate::lint::{Severity, Violation};
use crate::rule::{Rule, RuleCategory};
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
mod ignore_good;
#[cfg(test)]
mod generated_fix;