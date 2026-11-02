use std::sync::OnceLock;

use nu_lint_analysis::{
    LintLevel,
    ast::{
        lexed_statement::literal_text_spans,
        source_line::{SourceLine, file_source_lines, line_index_at},
    },
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
    static WHITESPACE_PATTERN: OnceLock<Regex> = OnceLock::new();
    WHITESPACE_PATTERN.get_or_init(|| Regex::new(r"[\t\u{a0}]+").unwrap())
}

enum WhitespaceRewrite {
    IndentationRun {
        whitespace_span: Span,
        character_count: usize,
    },
    InlineGap(Span),
}

fn classify_whitespace_run(
    whitespace_span: Span,
    character_count: usize,
    source_lines: &[SourceLine],
) -> Option<WhitespaceRewrite> {
    let source_line = &source_lines[line_index_at(source_lines, whitespace_span.start)];
    if whitespace_span.end >= source_line.visible_end {
        None
    } else if whitespace_span.start <= source_line.content_start() {
        Some(WhitespaceRewrite::IndentationRun {
            whitespace_span,
            character_count,
        })
    } else {
        Some(WhitespaceRewrite::InlineGap(whitespace_span))
    }
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
        let source_text = unsafe { context.source() };
        let file_offset = context.file_offset();
        let literal_spans = literal_text_spans(context);
        let source_lines = file_source_lines(context);

        pasted_whitespace_pattern()
            .find_iter(source_text)
            .filter_map(|whitespace_match| {
                let whitespace_span = Span::new(
                    whitespace_match.start() + file_offset,
                    whitespace_match.end() + file_offset,
                );
                let is_literal_text = literal_spans
                    .iter()
                    .any(|literal_span| literal_span.contains_span(whitespace_span));
                let character_count = whitespace_match.as_str().chars().count();
                (!is_literal_text)
                    .then(|| {
                        classify_whitespace_run(whitespace_span, character_count, &source_lines)
                    })
                    .flatten()
                    .map(|whitespace_rewrite| {
                        (
                            Detection::from_global_span(
                                "Tab or non-breaking space outside a string",
                                whitespace_span,
                            )
                            .with_primary_label("pasted whitespace"),
                            whitespace_rewrite,
                        )
                    })
            })
            .collect()
    }

    fn fix(
        &self,
        _lint_context: &LintContext,
        whitespace_rewrite: &Self::FixInput<'_>,
    ) -> Option<Fix> {
        let replacement = match whitespace_rewrite {
            WhitespaceRewrite::IndentationRun {
                whitespace_span,
                character_count,
            } => Replacement::new(
                *whitespace_span,
                " ".repeat(character_count * INDENT_WIDTH_PER_CHARACTER),
            ),
            WhitespaceRewrite::InlineGap(whitespace_span) => {
                Replacement::new(*whitespace_span, " ".to_string())
            }
        };
        Some(Fix {
            explanation: "Replace with plain spaces".into(),
            replacements: vec![replacement],
        })
    }
}

pub static RULE: &dyn Rule = &NormalizePastedWhitespace;
