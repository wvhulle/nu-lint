use std::sync::OnceLock;

use regex::Regex;

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn each_if_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"each\s*\{\s*\|([^}|]+)\|\s*if\s+([^}]+)\}").unwrap())
}

// Side effects that indicate this is processing, not filtering
const SIDE_EFFECTS: &[&str] = &["print", "save", "download", "^", "exit", "=", "mut "];

fn has_side_effects(code: &str) -> bool {
    SIDE_EFFECTS.iter().any(|&effect| code.contains(effect))
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context.violations_from_regex(each_if_pattern(), "prefer_where_over_each_if", |mat| {
        let caps = each_if_pattern().captures(mat.as_str())?;
        let condition_and_body = caps.get(2)?.as_str();

        let body = condition_and_body
            .find('{')
            .map_or(condition_and_body, |pos| &condition_and_body[pos + 1..])
            .trim();

        // Check if this is pure filtering (no side effects)
        (!has_side_effects(body)).then(|| {
            (
                "Consider using 'where' for filtering instead of 'each' with 'if'".to_string(),
                Some("Use '$list | where <condition>' for better performance".to_string()),
            )
        })
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_where_over_each_if",
        RuleCategory::Idioms,
        Severity::Warning,
        "Use 'where' for filtering instead of 'each' with 'if'",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
