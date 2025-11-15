use crate::{LintLevel, context::LintContext, rule::Rule, violation::RuleViolation};

const fn check(_context: &LintContext) -> Vec<RuleViolation> {
    // This rule doesn't perform traditional checking.
    // Parse errors are extracted directly from the StateWorkingSet
    // and converted to violations in the engine.
    vec![]
}

pub fn rule() -> Rule {
    Rule::new(
        "nu_parse_error",
        LintLevel::Deny,
        "Nushell parser encountered a syntax error",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
#[cfg(test)]
mod suggestions;
