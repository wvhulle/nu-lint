use std::collections::HashSet;

use miette::Diagnostic;
use nu_protocol::ParseError;

use crate::{
    context::LintContext,
    rule::Rule,
    violation::{Label, Violation},
};

const NU_PARSER_VERSION: &str = env!("NU_PARSER_VERSION");

fn extract_labels(parse_error: &ParseError) -> Vec<Label> {
    parse_error
        .labels()
        .into_iter()
        .flatten()
        .map(|miette_label| {
            let span = nu_protocol::Span::new(miette_label.offset(), miette_label.offset() + miette_label.len());
            match miette_label.label() {
                Some(text) if !text.is_empty() => Label::new(span, text.to_string()),
                _ => Label::span_only(span),
            }
        })
        .collect()
}

fn build_help_text(parse_error: &ParseError) -> String {
    let version_note = format!(
        "nu-lint expects Nushell {NU_PARSER_VERSION}. If your installed version differs, this may \
         cause false positives."
    );

    if let Some(help_text) = parse_error.help() {
        format!("{help_text}\n\n{version_note}")
    } else {
        version_note
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    let mut seen = HashSet::new();
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
                let labels = extract_labels(parse_error);
                Violation::new(parse_error.to_string(), parse_error.span())
                    .with_help(build_help_text(parse_error))
                    .with_labels(labels)
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
