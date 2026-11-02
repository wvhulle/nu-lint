use std::sync::OnceLock;

use nu_lint_analysis::{
    LintLevel,
    ast::lexed_statement::literal_text_spans,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::Span;
use regex::Regex;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;

const INDENT_WIDTH_PER_CHARACTER: usize = 4;

fn pasted_whitespace_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"[\t\u{a0}]+").unwrap())
}

struct WhitespaceRewrite {
    whitespace_span: Span,
    plain_spaces: String,
}

fn is_line_indentation(source_before: &str) -> bool {
    source_before
        .rsplit('\n')
        .next()
        .is_some_and(|line_prefix| line_prefix.bytes().all(|byte| matches!(byte, b' ' | b'\t')))
}

const fn is_line_end(source_after: &str) -> bool {
    matches!(
        source_after.as_bytes(),
        [] | [b'\n', ..] | [b'\r', b'\n', ..]
    )
}

struct NormalizePastedWhitespace;

impl DetectFix for NormalizePastedWhitespace {
    type FixInput<'a> = WhitespaceRewrite;

    fn id(&self) -> &'static str {
        "normalize_pasted_whitespace"
    }

    fn short_description(&self) -> &'static str {
        "Tab or non-breaking space outside strings"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Web pages often contain non-breaking spaces, which look like spaces but turn `^echo \
             a` into one command name. Tabs parse fine but differ from the spaces the style guide \
             uses. Text inside strings and comments is left alone.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/style_guide.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Off
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let source = unsafe { context.source() };
        let file_offset = context.file_offset();
        let literal_spans = literal_text_spans(context);

        pasted_whitespace_pattern()
            .find_iter(source)
            .filter(|whitespace| !is_line_end(&source[whitespace.end()..]))
            .filter_map(|whitespace| {
                let whitespace_span = Span::new(
                    whitespace.start() + file_offset,
                    whitespace.end() + file_offset,
                );
                if literal_spans
                    .iter()
                    .any(|literal_span| literal_span.contains_span(whitespace_span))
                {
                    return None;
                }
                let plain_spaces = if is_line_indentation(&source[..whitespace.start()]) {
                    " ".repeat(whitespace.as_str().chars().count() * INDENT_WIDTH_PER_CHARACTER)
                } else {
                    " ".to_string()
                };
                Some((
                    Detection::from_global_span(
                        "Tab or non-breaking space outside a string",
                        whitespace_span,
                    )
                    .with_primary_label("pasted whitespace"),
                    WhitespaceRewrite {
                        whitespace_span,
                        plain_spaces,
                    },
                ))
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, rewrite: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix {
            explanation: "Replace with plain spaces".into(),
            replacements: vec![Replacement::new(
                rewrite.whitespace_span,
                rewrite.plain_spaces.clone(),
            )],
        })
    }
}

pub static RULE: &dyn Rule = &NormalizePastedWhitespace;
