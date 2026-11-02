use std::mem::take;

pub use nu_parser::{Token, TokenContents};
use nu_protocol::Span;

use crate::context::LintContext;

pub struct LexedStatement {
    pub statement_tokens: Vec<Token>,
}

#[derive(Default)]
struct StatementSplitter {
    completed_statements: Vec<Vec<Token>>,
    current_tokens: Vec<Token>,
}

impl StatementSplitter {
    fn finish_statement(&mut self) {
        self.completed_statements
            .push(take(&mut self.current_tokens));
    }

    fn is_line_continued(&self, lint_context: &LintContext) -> bool {
        self.current_tokens
            .last()
            .is_some_and(|last_token| continues_after_newline(last_token, lint_context))
    }
}

#[must_use]
pub fn token_text_equals(
    lexed_token: &Token,
    lint_context: &LintContext,
    expected_text: &[u8],
) -> bool {
    token_source_bytes(lexed_token, lint_context) == expected_text
}

#[must_use]
pub fn token_source_bytes<'a>(lexed_token: &Token, lint_context: &'a LintContext) -> &'a [u8] {
    lint_context.working_set.get_span_contents(lexed_token.span)
}

#[must_use]
pub fn user_file_span(lint_context: &LintContext) -> Span {
    let file_start = lint_context.file_offset();
    Span::new(file_start, file_start + lint_context.source_len())
}

fn lex_span_tokens(lexed_span: Span, lint_context: &LintContext) -> Vec<Token> {
    nu_parser::lex(
        lint_context.working_set.get_span_contents(lexed_span),
        lexed_span.start,
        &[],
        &[],
        false,
    )
    .0
}

fn delimited_interior(lexed_token: &Token, lint_context: &LintContext) -> Option<Span> {
    if lexed_token.contents != TokenContents::Item {
        return None;
    }
    let token_bytes = token_source_bytes(lexed_token, lint_context);
    let closing_delimiter = match token_bytes.first()? {
        b'{' => b'}',
        b'(' => b')',
        b'[' => b']',
        _ => return None,
    };
    (token_bytes.len() >= 2 && token_bytes.last() == Some(&closing_delimiter))
        .then(|| Span::new(lexed_token.span.start + 1, lexed_token.span.end - 1))
}

fn continues_after_newline(lexed_token: &Token, lint_context: &LintContext) -> bool {
    matches!(
        lexed_token.contents,
        TokenContents::Pipe
            | TokenContents::PipePipe
            | TokenContents::ErrGreaterPipe
            | TokenContents::OutErrGreaterPipe
    ) || token_text_equals(lexed_token, lint_context, b"&&")
}

#[must_use]
pub fn is_pipe_token(lexed_token: &Token, lint_context: &LintContext) -> bool {
    token_text_equals(lexed_token, lint_context, b"|")
}

fn closure_parameter_count(lexed_tokens: &[Token], lint_context: &LintContext) -> usize {
    match lexed_tokens.first() {
        Some(first_token) if first_token.contents == TokenContents::PipePipe => 1,
        Some(first_token) if is_pipe_token(first_token, lint_context) => lexed_tokens
            .iter()
            .skip(1)
            .position(|lexed_token| is_pipe_token(lexed_token, lint_context))
            .map_or(0, |closing_position| closing_position + 2),
        _ => 0,
    }
}

fn split_into_statements(
    lexed_tokens: Vec<Token>,
    lint_context: &LintContext,
) -> Vec<LexedStatement> {
    let mut statement_splitter = lexed_tokens.into_iter().fold(
        StatementSplitter::default(),
        |mut statement_splitter, lexed_token| {
            match lexed_token.contents {
                TokenContents::Comment => {}
                TokenContents::Semicolon => statement_splitter.finish_statement(),
                TokenContents::Eol => {
                    if !statement_splitter.is_line_continued(lint_context) {
                        statement_splitter.finish_statement();
                    }
                }
                _ => statement_splitter.current_tokens.push(lexed_token),
            }
            statement_splitter
        },
    );
    statement_splitter.finish_statement();

    statement_splitter
        .completed_statements
        .into_iter()
        .filter(|statement_tokens| !statement_tokens.is_empty())
        .map(|statement_tokens| LexedStatement { statement_tokens })
        .collect()
}

fn nested_statements(lexed_span: Span, lint_context: &LintContext) -> Vec<LexedStatement> {
    let mut lexed_tokens = lex_span_tokens(lexed_span, lint_context);
    let mut gathered_statements: Vec<LexedStatement> = lexed_tokens
        .iter()
        .filter_map(|lexed_token| delimited_interior(lexed_token, lint_context))
        .flat_map(|interior_span| nested_statements(interior_span, lint_context))
        .collect();
    lexed_tokens.drain(..closure_parameter_count(&lexed_tokens, lint_context));
    gathered_statements.extend(split_into_statements(lexed_tokens, lint_context));
    gathered_statements
}

#[must_use]
pub fn all_lexed_statements(lint_context: &LintContext) -> Vec<LexedStatement> {
    nested_statements(user_file_span(lint_context), lint_context)
}

#[must_use]
pub fn is_quoted_string_token(lexed_token: &Token, lint_context: &LintContext) -> bool {
    lexed_token.contents == TokenContents::Item
        && matches!(
            token_source_bytes(lexed_token, lint_context),
            [b'\'' | b'"' | b'`', ..] | [b'$', b'\'' | b'"', ..] | [b'r', b'#', ..]
        )
}

fn literal_spans_within(lexed_span: Span, lint_context: &LintContext) -> Vec<Span> {
    lex_span_tokens(lexed_span, lint_context)
        .iter()
        .flat_map(|lexed_token| {
            if lexed_token.contents == TokenContents::Comment
                || is_quoted_string_token(lexed_token, lint_context)
            {
                vec![lexed_token.span]
            } else {
                delimited_interior(lexed_token, lint_context)
                    .map_or_else(Vec::new, |interior_span| {
                        literal_spans_within(interior_span, lint_context)
                    })
            }
        })
        .collect()
}

#[must_use]
pub fn literal_text_spans(lint_context: &LintContext) -> Vec<Span> {
    literal_spans_within(user_file_span(lint_context), lint_context)
}
