use nu_lint_analysis::{
    ast::lexed_statement::{
        LexedStatement, Token, TokenContents, is_pipe_token, token_source_bytes, token_text_equals,
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
    UnclearOperands,
}

fn is_logical_operator(lexed_token: &Token, lint_context: &LintContext) -> bool {
    lexed_token.contents == TokenContents::PipePipe
        || token_text_equals(lexed_token, lint_context, b"&&")
}

fn starts_external_command(lexed_token: &Token, lint_context: &LintContext) -> bool {
    token_source_bytes(lexed_token, lint_context).first() == Some(&b'^')
}

fn classify_operand_context(
    lexed_statement: &LexedStatement,
    lint_context: &LintContext,
) -> OperandContext {
    let lexed_tokens = &lexed_statement.statement_tokens;
    let first_operator_index = lexed_tokens
        .iter()
        .position(|lexed_token| is_logical_operator(lexed_token, lint_context))
        .unwrap_or(lexed_tokens.len());
    let operand_head = lexed_tokens[..first_operator_index]
        .iter()
        .rev()
        .take_while(|lexed_token| !is_pipe_token(lexed_token, lint_context))
        .last();
    let right_operand_runs_command = lexed_tokens
        .get(first_operator_index + 1)
        .is_some_and(|right_operand| starts_external_command(right_operand, lint_context));

    let Some(head_token) = operand_head else {
        return OperandContext::CommandChain;
    };
    let head_bytes = token_source_bytes(head_token, lint_context);
    let reads_as_boolean =
        head_bytes.first() == Some(&b'$') || BOOLEAN_CONTEXT_KEYWORDS.contains(&head_bytes);

    match head_bytes.first() {
        Some(b'[' | b'(' | b'"' | b'\'' | b'0'..=b'9') => OperandContext::UnclearOperands,
        _ if reads_as_boolean && right_operand_runs_command => OperandContext::UnclearOperands,
        _ if reads_as_boolean => OperandContext::BooleanExpression,
        _ => OperandContext::CommandChain,
    }
}
