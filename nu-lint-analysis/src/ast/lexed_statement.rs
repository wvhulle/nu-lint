use std::mem::take;

pub use nu_parser::{Token, TokenContents};
use nu_protocol::Span;

use crate::context::LintContext;

pub struct LexedStatement {
    pub statement_tokens: Vec<Token>,
}

impl LexedStatement {
    #[must_use]
    pub fn full_span(&self) -> Option<Span> {
        let first_token = self.statement_tokens.first()?;
        let last_token = self.statement_tokens.last()?;
        Some(Span::new(first_token.span.start, last_token.span.end))
    }
}

#[must_use]
pub fn token_text_equals(token: &Token, context: &LintContext, expected_text: &[u8]) -> bool {
    context.working_set.get_span_contents(token.span) == expected_text
}

fn user_file_span(context: &LintContext) -> Span {
    let file_start = context.file_offset();
    Span::new(file_start, file_start + context.source_len())
}

fn lex_span_tokens(span: Span, context: &LintContext) -> Vec<Token> {
    nu_parser::lex(
        context.working_set.get_span_contents(span),
        span.start,
        &[],
        &[],
        false,
    )
    .0
}

fn delimited_interior(token: &Token, context: &LintContext) -> Option<Span> {
    if token.contents != TokenContents::Item {
        return None;
    }
    let contents = context.working_set.get_span_contents(token.span);
    let closing_delimiter = match contents.first()? {
        b'{' => b'}',
        b'(' => b')',
        b'[' => b']',
        _ => return None,
    };
    (contents.len() >= 2 && contents.last() == Some(&closing_delimiter))
        .then(|| Span::new(token.span.start + 1, token.span.end - 1))
}

fn continues_after_newline(token: &Token, context: &LintContext) -> bool {
    matches!(
        token.contents,
        TokenContents::Pipe
            | TokenContents::PipePipe
            | TokenContents::ErrGreaterPipe
            | TokenContents::OutErrGreaterPipe
    ) || token_text_equals(token, context, b"&&")
}

#[must_use]
pub fn is_pipe_token(token: &Token, context: &LintContext) -> bool {
    token_text_equals(token, context, b"|")
}

fn closure_parameter_count(tokens: &[Token], context: &LintContext) -> usize {
    match tokens.first() {
        Some(first_token) if first_token.contents == TokenContents::PipePipe => 1,
        Some(first_token) if is_pipe_token(first_token, context) => tokens
            .iter()
            .skip(1)
            .position(|token| is_pipe_token(token, context))
            .map_or(0, |closing_position| closing_position + 2),
        _ => 0,
    }
}

fn split_into_statements(tokens: Vec<Token>, context: &LintContext) -> Vec<LexedStatement> {
    let mut completed_statements = Vec::new();
    let mut current_tokens: Vec<Token> = Vec::new();

    for token in tokens {
        match token.contents {
            TokenContents::Comment => {}
            TokenContents::Semicolon => {
                completed_statements.push(take(&mut current_tokens));
            }
            TokenContents::Eol => {
                let line_continues = current_tokens
                    .last()
                    .is_some_and(|last_token| continues_after_newline(last_token, context));
                if !line_continues {
                    completed_statements.push(take(&mut current_tokens));
                }
            }
            _ => current_tokens.push(token),
        }
    }
    completed_statements.push(current_tokens);

    completed_statements
        .into_iter()
        .filter(|statement_tokens| !statement_tokens.is_empty())
        .map(|statement_tokens| LexedStatement { statement_tokens })
        .collect()
}

fn collect_nested_statements(
    span: Span,
    context: &LintContext,
    statements: &mut Vec<LexedStatement>,
) {
    let mut tokens = lex_span_tokens(span, context);
    for interior_span in tokens
        .iter()
        .filter_map(|token| delimited_interior(token, context))
    {
        collect_nested_statements(interior_span, context, statements);
    }
    tokens.drain(..closure_parameter_count(&tokens, context));
    statements.extend(split_into_statements(tokens, context));
}

#[must_use]
pub fn all_lexed_statements(context: &LintContext) -> Vec<LexedStatement> {
    let mut statements = Vec::new();
    collect_nested_statements(user_file_span(context), context, &mut statements);
    statements
}

#[must_use]
pub fn top_level_statements(context: &LintContext) -> Vec<LexedStatement> {
    split_into_statements(lex_span_tokens(user_file_span(context), context), context)
}

#[must_use]
pub fn is_quoted_string_token(token: &Token, context: &LintContext) -> bool {
    if token.contents != TokenContents::Item {
        return false;
    }
    matches!(
        context.working_set.get_span_contents(token.span),
        [b'\'' | b'"' | b'`', ..] | [b'$', b'\'' | b'"', ..] | [b'r', b'#', ..]
    )
}

#[must_use]
pub fn is_raw_string_token(token: &Token, context: &LintContext) -> bool {
    matches!(
        context.working_set.get_span_contents(token.span),
        [b'r', b'#', ..]
    )
}

fn collect_literal_text_spans(span: Span, context: &LintContext, literal_spans: &mut Vec<Span>) {
    for token in lex_span_tokens(span, context) {
        if token.contents == TokenContents::Comment || is_quoted_string_token(&token, context) {
            literal_spans.push(token.span);
        } else if let Some(interior_span) = delimited_interior(&token, context) {
            collect_literal_text_spans(interior_span, context, literal_spans);
        }
    }
}

#[must_use]
pub fn literal_text_spans(context: &LintContext) -> Vec<Span> {
    let mut literal_spans = Vec::new();
    collect_literal_text_spans(user_file_span(context), context, &mut literal_spans);
    literal_spans
}
