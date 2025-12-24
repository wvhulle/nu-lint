use std::collections::HashSet;

use miette::Diagnostic;
use nu_protocol::ParseError;

use crate::{LintLevel, NU_PARSER_VERSION, context::LintContext, rule::Rule, violation::Violation};

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
            if !seen.insert(key) {
                return None;
            }

            let mut violation = Violation::new(parse_error.to_string(), parse_error.span())
                .with_help(build_help_text(parse_error));

            // Add extra labels from parse error
            let labels: Vec<_> = parse_error.labels().into_iter().flatten().collect();
            for label in labels {
                let span = nu_protocol::Span::new(label.offset(), label.offset() + label.len());
                let label_text = label.label().map(ToString::to_string);
                violation = match label_text {
                    Some(text) => violation.with_extra_label(text, span),
                    None => violation.with_extra_span(span),
                };
            }

            Some(violation)
        })
        .collect()
}
pub const RULE: Rule = Rule::new(
    "nu_parse_error",
    "Nushell parser encountered a syntax error",
    check,
    LintLevel::Error,
)
.with_doc_url("https://www.nushell.sh/blog/");
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
