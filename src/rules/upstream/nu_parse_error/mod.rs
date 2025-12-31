use std::str::from_utf8;

use miette::Diagnostic;
use nu_protocol::{ParseError, Span, engine::StateWorkingSet};

use crate::{
    LintLevel, NU_PARSER_VERSION,
    context::LintContext,
    rule::{DetectFix, Rule},
    span::FileSpan,
    violation::{Detection, ExternalDetection},
};

/// Information about a file in the working set
struct FileInfo<'a> {
    name: &'a str,
    content: &'a [u8],
    covered_span: Span,
}

fn find_file_for_span<'a>(working_set: &'a StateWorkingSet, span: Span) -> Option<FileInfo<'a>> {
    for file in working_set.files() {
        if file.covered_span.contains_span(span) {
            return Some(FileInfo {
                name: &file.name,
                content: &file.content,
                covered_span: file.covered_span,
            });
        }
    }
    None
}

fn build_help_text(parse_error: &ParseError) -> String {
    let version_note = format!(
        "nu-lint expects Nushell {NU_PARSER_VERSION}. If your installed version differs, this may \
         cause false positives."
    );

    let mut parts = Vec::new();

    if let Some(help_text) = parse_error.help() {
        parts.push(help_text.to_string());
    }

    parts.push(version_note);

    parts.join("\n\n")
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
        let mut detections = Vec::new();

        for parse_error in &context.working_set.parse_errors {
            let error_span = parse_error.span();
            let in_external_file = !context.span_in_user_file(error_span);

            if in_external_file && context.config.skip_external_parse_errors {
                log::debug!(
                    "Skipping parse error in external file: {parse_error:?} (span {error_span:?})",
                );
                continue;
            }
            log::debug!("Found parse error in user file: {parse_error:?}");

            // Collect labels
            let labels: Vec<_> = parse_error.labels().into_iter().flatten().collect();

            let mut detection = Detection::from_global_span(parse_error.to_string(), error_span)
                .with_help(build_help_text(parse_error));

            // Process labels - add in-file labels as extra_labels, external as
            // ExternalDetection
            for label in labels {
                let span = Span::new(label.offset(), label.offset() + label.len());

                if context.span_in_user_file(span) {
                    let label_text = label.label().map(ToString::to_string);
                    detection = match label_text {
                        Some(text) => detection.with_extra_label(text, span),
                        None => detection.with_extra_span(span),
                    };
                } else if let Some(file_info) = find_file_for_span(context.working_set, span) {
                    // Create ExternalDetection for labels pointing to external files
                    let file_relative_start = span.start - file_info.covered_span.start;
                    let file_relative_end = span.end - file_info.covered_span.start;
                    let file_span = FileSpan::new(file_relative_start, file_relative_end);

                    let source = from_utf8(file_info.content)
                        .unwrap_or("<invalid utf8>")
                        .to_string();

                    let label_text = label.label().unwrap_or("related code");
                    let external = ExternalDetection::new(
                        file_info.name,
                        source,
                        file_span,
                        format!("{parse_error}: {label_text}"),
                    )
                    .with_label(label_text.to_string());

                    detection = detection.with_external_detection(external);
                }
            }

            detections.push(detection);
        }

        Self::no_fix(detections)
    }
}

pub static RULE: &dyn Rule = &NuParseError;
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
