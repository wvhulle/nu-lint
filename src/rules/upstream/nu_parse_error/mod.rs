use std::collections::HashSet;

use miette::Diagnostic;
use nu_protocol::ParseError;

use crate::{
    LintLevel, NU_PARSER_VERSION,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

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

struct NuParseError;

impl DetectFix for NuParseError {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "nu_parse_error"
    }

    fn explanation(&self) -> &'static str {
        "Nushell parser encountered a syntax error"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/blog/")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Error
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut seen = HashSet::new();
        Self::no_fix(
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

                    let mut violation =
                        Detection::from_global_span(parse_error.to_string(), parse_error.span())
                            .with_help(build_help_text(parse_error));

                    // Add extra labels from parse error
                    let labels: Vec<_> = parse_error.labels().into_iter().flatten().collect();
                    for label in labels {
                        let span =
                            nu_protocol::Span::new(label.offset(), label.offset() + label.len());
                        let label_text = label.label().map(ToString::to_string);
                        violation = match label_text {
                            Some(text) => violation.with_extra_label(text, span),
                            None => violation.with_extra_span(span),
                        };
                    }

                    Some(violation)
                })
                .collect(),
        )
    }
}

pub static RULE: &dyn Rule = &NuParseError;
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
