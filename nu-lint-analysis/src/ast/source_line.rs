use nu_protocol::Span;

use crate::{ast::lexed_statement::user_file_span, context::LintContext};

pub struct SourceLine {
    pub line_start: usize,
    pub visible_end: usize,
    pub next_line_start: usize,
    pub indent_width: usize,
}

impl SourceLine {
    #[must_use]
    pub const fn content_start(&self) -> usize {
        self.line_start + self.indent_width
    }
}

const fn parse_source_line(line_start: usize, line_bytes: &[u8]) -> SourceLine {
    let visible_bytes = line_bytes.trim_ascii_end();
    SourceLine {
        line_start,
        visible_end: line_start + visible_bytes.len(),
        next_line_start: line_start + line_bytes.len(),
        indent_width: visible_bytes.len() - visible_bytes.trim_ascii_start().len(),
    }
}

#[must_use]
pub fn file_source_lines(lint_context: &LintContext) -> Vec<SourceLine> {
    let file_span = user_file_span(lint_context);
    lint_context
        .working_set
        .get_span_contents(file_span)
        .split_inclusive(|file_byte| *file_byte == b'\n')
        .scan(file_span.start, |next_line_start, line_bytes| {
            let source_line = parse_source_line(*next_line_start, line_bytes);
            *next_line_start = source_line.next_line_start;
            Some(source_line)
        })
        .collect()
}

#[must_use]
pub fn line_index_at(source_lines: &[SourceLine], global_position: usize) -> usize {
    source_lines
        .partition_point(|source_line| source_line.next_line_start <= global_position)
        .min(source_lines.len().saturating_sub(1))
}

#[must_use]
pub fn has_inline_doc_comment(declaration_span: Span, context: &LintContext) -> bool {
    let after_declaration = context.source_after_span(declaration_span);
    let line_end = after_declaration
        .find('\n')
        .unwrap_or(after_declaration.len());
    after_declaration[..line_end].contains(" # ")
}
