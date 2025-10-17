use std::sync::OnceLock;

use regex::Regex;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
};

#[derive(Default)]
pub struct PreferWhereOverEachIf;

impl PreferWhereOverEachIf {
    fn each_if_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"each\s*\{\s*\|([^}|]+)\|\s*if\s+([^}]+)\}").unwrap())
    }

    // Side effects that indicate this is processing, not filtering
    const SIDE_EFFECTS: &'static [&'static str] =
        &["print", "save", "download", "^", "exit", "=", "mut "];
}

impl Rule for PreferWhereOverEachIf {
    fn id(&self) -> &'static str {
        "prefer_where_over_each_if"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Performance
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &'static str {
        "Use 'where' for filtering instead of 'each' with 'if'"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        context.violations_from_regex_if(
            Self::each_if_pattern(),
            self.id(),
            self.severity(),
            |mat| {
                let caps = Self::each_if_pattern().captures(mat.as_str())?;
                let condition_and_body = caps.get(2)?.as_str();

                let body = condition_and_body
                    .find('{')
                    .map_or(condition_and_body, |pos| &condition_and_body[pos + 1..])
                    .trim();

                // Check if this is pure filtering (no side effects)
                (!Self::has_side_effects(body)).then(|| {
                    (
                        "Consider using 'where' for filtering instead of 'each' with 'if'"
                            .to_string(),
                        Some("Use '$list | where <condition>' for better performance".to_string()),
                    )
                })
            },
        )
    }
}

impl PreferWhereOverEachIf {
    fn has_side_effects(code: &str) -> bool {
        Self::SIDE_EFFECTS
            .iter()
            .any(|&effect| code.contains(effect))
    }
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
