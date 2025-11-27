use std::collections::HashSet;

use miette::Diagnostic;
use nu_protocol::ParseError;

use crate::{context::LintContext, rule::Rule, violation::Violation};

fn extract_label_texts(parse_error: &ParseError) -> Vec<String> {
    parse_error
        .labels()
        .into_iter()
        .flatten()
        .filter_map(|label| {
            let text = label.label()?;
            (!text.is_empty()).then(|| text.to_string())
        })
        .collect()
}

fn build_help_text(parse_error: &ParseError) -> Option<String> {
    // Prefer the help text from ParseError if available, as it's usually more
    // comprehensive
    if let Some(help_text) = parse_error.help() {
        return Some(help_text.to_string());
    }

    // Fall back to label texts if no help is available
    // Labels often contain useful context about what was expected or what went
    // wrong
    let labels = extract_label_texts(parse_error);
    (!labels.is_empty()).then(|| labels.join("\n"))
}

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
                let mut violation = Violation::new(parse_error.to_string(),
                    parse_error.span(),
                );

                if let Some(help) = build_help_text(parse_error) {
                    violation = violation.with_help(help);
                }

                violation
            })
        })
        .collect()
}
pub const fn rule() -> Rule {
    Rule::new(
        "nu_parse_error",
        "Nushell parser encountered a syntax error",
        check,
    )
    .with_doc_url("https://www.nushell.sh/book/")
}
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
