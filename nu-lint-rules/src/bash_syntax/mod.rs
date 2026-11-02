use nu_lint_analysis::{
    ast::lexed_statement::{
        LexedStatement, Token, TokenContents, is_pipe_token, token_text_equals,
    },
    context::LintContext,
};

pub mod shell_and_to_semicolon;
pub mod shell_line_continuation;
pub mod shell_or_to_try;
pub mod shell_redirect_to_nu;
pub mod shell_variable_to_env;

const BOOLEAN_CONTEXT_KEYWORDS: &[&[u8]] = &[
    b"if", b"while", b"where", b"let", b"mut", b"const", b"return", b"not", b"true", b"false",
];

enum OperandContext {
    CommandChain,
    BooleanExpression,
    BashTest,
}

fn is_logical_operator(token: &Token, context: &LintContext) -> bool {
    token.contents == TokenContents::PipePipe || token_text_equals(token, context, b"&&")
}

fn classify_operand_context(statement: &LexedStatement, context: &LintContext) -> OperandContext {
    let tokens = &statement.statement_tokens;
    let first_operator_index = tokens
        .iter()
        .position(|token| is_logical_operator(token, context))
        .unwrap_or(tokens.len());
    let operand_head = tokens[..first_operator_index]
        .iter()
        .rev()
        .take_while(|token| !is_pipe_token(token, context))
        .last();

    let Some(head_token) = operand_head else {
        return OperandContext::CommandChain;
    };
    let head_contents = context.working_set.get_span_contents(head_token.span);

    match head_contents.first() {
        Some(b'[') => OperandContext::BashTest,
        Some(b'$' | b'(' | b'"' | b'\'' | b'0'..=b'9') => OperandContext::BooleanExpression,
        _ if BOOLEAN_CONTEXT_KEYWORDS.contains(&head_contents) => OperandContext::BooleanExpression,
        _ => OperandContext::CommandChain,
    }
}
