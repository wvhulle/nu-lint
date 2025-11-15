use std::collections::HashSet;

use nu_protocol::ParseError;

use crate::{context::LintContext, rule::Rule, violation::Violation};
fn check(context: &LintContext) -> Vec<Violation> {
    let mut seen = HashSet::new();
    // Convert each parse error to a violation, deduplicating by span and message
    // Filter out module-related errors since the linter works at AST level only
    context
        .working_set
        .parse_errors
        .iter()
        .filter(|parse_error| !matches!(parse_error, ParseError::ModuleNotFound(_, _)))
        .filter_map(|parse_error| {
            let key = (
                parse_error.span().start,
                parse_error.span().end,
                parse_error.to_string(),
            );
            seen.insert(key).then(|| {
                Violation::new_dynamic(
                    "nu_parse_error",
                    parse_error.to_string(),
                    parse_error.span(),
                )
            })
        })
        .collect()
}
pub fn rule() -> Rule {
    Rule::new(
        "nu_parse_error",
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
