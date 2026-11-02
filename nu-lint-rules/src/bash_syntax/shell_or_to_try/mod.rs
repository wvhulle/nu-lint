use std::iter::once;

use nu_lint_analysis::{
    LintLevel,
    ast::lexed_statement::{LexedStatement, Token, TokenContents, all_lexed_statements},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::Span;

use super::{OperandContext, classify_operand_context};

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;

struct TryCatchSpans {
    first_operand_start: usize,
    operand_gaps: Vec<Span>,
    last_operand_end: usize,
}

enum OrOperatorRewrite {
    BooleanKeyword(Span),
    TryCatchNesting(TryCatchSpans),
    NoEquivalent,
}

struct OrOperatorOccurrence {
    operator_span: Span,
    operator_rewrite: OrOperatorRewrite,
}

fn is_or_operator(lexed_token: &Token) -> bool {
    lexed_token.contents == TokenContents::PipePipe
}

fn try_catch_spans(lexed_tokens: &[Token]) -> Option<TryCatchSpans> {
    let first_token = lexed_tokens
        .first()
        .filter(|lexed_token| !is_or_operator(lexed_token))?;
    let last_token = lexed_tokens
        .last()
        .filter(|lexed_token| !is_or_operator(lexed_token))?;
    let has_adjacent_operators = lexed_tokens
        .windows(2)
        .any(|token_pair| token_pair.iter().all(is_or_operator));
    if has_adjacent_operators {
        return None;
    }

    Some(TryCatchSpans {
        first_operand_start: first_token.span.start,
        operand_gaps: (1..lexed_tokens.len() - 1)
            .filter(|&token_index| is_or_operator(&lexed_tokens[token_index]))
            .map(|token_index| {
                Span::new(
                    lexed_tokens[token_index - 1].span.end,
                    lexed_tokens[token_index + 1].span.start,
                )
            })
            .collect(),
        last_operand_end: last_token.span.end,
    })
}

fn statement_or_operators(
    lexed_statement: &LexedStatement,
    lint_context: &LintContext,
) -> Vec<OrOperatorOccurrence> {
    let lexed_tokens = &lexed_statement.statement_tokens;
    let or_operators = lexed_tokens
        .iter()
        .filter(|lexed_token| is_or_operator(lexed_token));

    match classify_operand_context(lexed_statement, lint_context) {
        OperandContext::BooleanExpression => or_operators
            .map(|or_operator| OrOperatorOccurrence {
                operator_span: or_operator.span,
                operator_rewrite: OrOperatorRewrite::BooleanKeyword(or_operator.span),
            })
            .collect(),
        OperandContext::UnclearOperands => or_operators
            .map(|or_operator| OrOperatorOccurrence {
                operator_span: or_operator.span,
                operator_rewrite: OrOperatorRewrite::NoEquivalent,
            })
            .collect(),
        OperandContext::CommandChain => lexed_tokens
            .iter()
            .find(|lexed_token| is_or_operator(lexed_token))
            .map(|first_operator| OrOperatorOccurrence {
                operator_span: first_operator.span,
                operator_rewrite: try_catch_spans(lexed_tokens).map_or(
                    OrOperatorRewrite::NoEquivalent,
                    OrOperatorRewrite::TryCatchNesting,
                ),
            })
            .into_iter()
            .collect(),
    }
}

fn insertion_at(insert_position: usize, inserted_text: String) -> Replacement {
    Replacement::new(Span::new(insert_position, insert_position), inserted_text)
}

fn catch_opening(operand_gap: Span) -> Replacement {
    Replacement::new(operand_gap, " } catch { ".to_string())
}

fn try_catch_replacements(nesting_spans: &TryCatchSpans) -> Option<Vec<Replacement>> {
    let (last_gap, earlier_gaps) = nesting_spans.operand_gaps.split_last()?;
    Some(
        once(insertion_at(
            nesting_spans.first_operand_start,
            "try { ".to_string(),
        ))
        .chain(earlier_gaps.iter().flat_map(|operand_gap| {
            [
                catch_opening(*operand_gap),
                insertion_at(operand_gap.end, "try { ".to_string()),
            ]
        }))
        .chain(once(catch_opening(*last_gap)))
        .chain(once(insertion_at(
            nesting_spans.last_operand_end,
            " }".repeat(nesting_spans.operand_gaps.len()),
        )))
        .collect(),
    )
}

struct ShellOrToTry;

impl DetectFix for ShellOrToTry {
    type FixInput<'a> = OrOperatorRewrite;

    fn id(&self) -> &'static str {
        "shell_or_to_try"
    }

    fn short_description(&self) -> &'static str {
        "Bash `||` replaceable with `try`/`catch` or `or`"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "There is no `||` operator. Use `try { a } catch { b }` to run a fallback command, \
             and `or` to combine booleans. There is no fix when the operands could be either \
             commands or booleans, such as after a bash test `[ -f x ]`.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/coming_from_bash.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Error
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        all_lexed_statements(context)
            .iter()
            .flat_map(|lexed_statement| statement_or_operators(lexed_statement, context))
            .map(|operator_occurrence| {
                (
                    Detection::from_global_span(
                        "The `||` operator does not exist in Nu",
                        operator_occurrence.operator_span,
                    )
                    .with_primary_label("bash operator"),
                    operator_occurrence.operator_rewrite,
                )
            })
            .collect()
    }

    fn fix(
        &self,
        _lint_context: &LintContext,
        operator_rewrite: &Self::FixInput<'_>,
    ) -> Option<Fix> {
        match operator_rewrite {
            OrOperatorRewrite::BooleanKeyword(operator_span) => Some(Fix {
                explanation: "Use the `or` operator".into(),
                replacements: vec![Replacement::new(*operator_span, "or".to_string())],
            }),
            OrOperatorRewrite::TryCatchNesting(nesting_spans) => Some(Fix {
                explanation: "Run the fallback with `try`/`catch`".into(),
                replacements: try_catch_replacements(nesting_spans)?,
            }),
            OrOperatorRewrite::NoEquivalent => None,
        }
    }
}

pub static RULE: &dyn Rule = &ShellOrToTry;
