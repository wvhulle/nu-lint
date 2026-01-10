use std::{collections::HashSet, str::from_utf8};

use const_format::formatcp;
use miette::Diagnostic;
use nu_protocol::{Span, engine::StateWorkingSet};

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

struct NuParseError;

impl DetectFix for NuParseError {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "nu_parse_error"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(formatcp!(
            "nu-lint expects Nushell {NU_PARSER_VERSION}. If your installed version differs, this \
             may cause false positives. Check that your Nushell version matches the expected version \
             to avoid incorrect warnings."
        ))
    }

    fn short_description(&self) -> &'static str {
        "Nushell parser encountered a syntax error"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/blog/")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Error
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut detections = Vec::new();
        let mut seen_errors = HashSet::new();

        for parse_error in &context.working_set.parse_errors {
            let error_span = parse_error.span();
            let in_external_file = !context.span_in_user_file(error_span);

            if in_external_file && context.config.skip_external_parse_errors {
                log::debug!(
                    "Skipping parse error in external file: {parse_error:?} (span {error_span:?})",
                );
                continue;
            }

            let error_key = (error_span.start, error_span.end, parse_error.to_string());
            if seen_errors.contains(&error_key) {
                log::debug!(
                    "Skipping duplicate parse error: {parse_error:?} (span {error_span:?})"
                );
                continue;
            }
            seen_errors.insert(error_key);

            log::debug!("Found parse error in user file: {parse_error:?}");

            // Collect labels
            let labels: Vec<_> = parse_error.labels().into_iter().flatten().collect();

            // Check if any labels point to external files (indicates dynamic import issue)
            let _has_external_labels = labels.iter().any(|label| {
                let span = Span::new(label.offset(), label.offset() + label.len());
                !context.span_in_user_file(span)
            });

            let mut detection = Detection::from_global_span(parse_error.to_string(), error_span);

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
mod ignore_good;
