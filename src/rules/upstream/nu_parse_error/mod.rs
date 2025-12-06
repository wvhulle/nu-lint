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

const NU_PARSER_VERSION: &str = env!("NU_PARSER_VERSION");

fn build_help_text(parse_error: &ParseError) -> String {
    let version_note = format!(
        "nu-lint expects Nushell {NU_PARSER_VERSION}. If your installed version differs, this may \
         cause false positives."
    );

    // Prefer the help text from ParseError if available, as it's usually more
    // comprehensive
    if let Some(help_text) = parse_error.help() {
        return format!("{help_text}\n\n{version_note}");
    }

    // Fall back to label texts if no help is available
    // Labels often contain useful context about what was expected or what went
    // wrong
    let labels = extract_label_texts(parse_error);
    if labels.is_empty() {
        version_note
    } else {
        format!("{}\n\n{version_note}", labels.join("\n"))
    }
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
                Violation::new(parse_error.to_string(), parse_error.span())
                    .with_help(build_help_text(parse_error))
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
    .with_doc_url("https://www.nushell.sh/blog/")
}
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
