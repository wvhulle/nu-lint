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

enum OrOperatorRewrite {
    BooleanKeyword(Span),
    TryCatchNesting(Vec<Replacement>),
    NoEquivalent,
}

fn is_or_operator(token: &Token) -> bool {
    token.contents == TokenContents::PipePipe
}

fn try_catch_edits(tokens: &[Token]) -> Option<Vec<Replacement>> {
    let first_token = tokens.first().filter(|token| !is_or_operator(token))?;
    let last_token = tokens.last().filter(|token| !is_or_operator(token))?;
    let operator_count = tokens.iter().filter(|token| is_or_operator(token)).count();
    let insertion_at = |position: usize, inserted_text: String| {
        Replacement::new(Span::new(position, position), inserted_text)
    };

    let operator_edits = tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| is_or_operator(token))
        .flat_map(|(token_index, _)| {
            let operand_end = tokens[token_index - 1].span.end;
            let next_operand_start = tokens[token_index + 1].span.start;
            let opening_try = tokens[token_index + 1..]
                .iter()
                .any(is_or_operator)
                .then(|| insertion_at(next_operand_start, "try { ".to_string()));
            once(Replacement::new(
                Span::new(operand_end, next_operand_start),
                " } catch { ".to_string(),
            ))
            .chain(opening_try)
        });

    Some(
        once(insertion_at(first_token.span.start, "try { ".to_string()))
            .chain(operator_edits)
            .chain(once(insertion_at(
                last_token.span.end,
                " }".repeat(operator_count),
            )))
            .collect(),
    )
}

fn statement_or_operators(
    statement: &LexedStatement,
    context: &LintContext,
) -> Vec<(Detection, OrOperatorRewrite)> {
    let tokens = &statement.statement_tokens;
    let operators = tokens.iter().filter(|token| is_or_operator(token));
    let operator_detection = |operator: &Token| {
        Detection::from_global_span("The `||` operator does not exist in Nu", operator.span)
            .with_primary_label("bash operator")
    };

    match classify_operand_context(statement, context) {
        OperandContext::BooleanExpression => operators
            .map(|operator| {
                (
                    operator_detection(operator),
                    OrOperatorRewrite::BooleanKeyword(operator.span),
                )
            })
            .collect(),
        OperandContext::BashTest => operators
            .map(|operator| {
                (
                    operator_detection(operator),
                    OrOperatorRewrite::NoEquivalent,
                )
            })
            .collect(),
        OperandContext::CommandChain => {
            let rewrite = try_catch_edits(tokens).map_or(
                OrOperatorRewrite::NoEquivalent,
                OrOperatorRewrite::TryCatchNesting,
            );
            tokens
                .iter()
                .find(|token| is_or_operator(token))
                .map(|first_operator| (operator_detection(first_operator), rewrite))
                .into_iter()
                .collect()
        }
    }
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
             and `or` to combine booleans. A bash test such as `[ -f x ] || y` gets no fix.",
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
            .flat_map(|statement| statement_or_operators(statement, context))
            .collect()
    }

    fn fix(&self, _context: &LintContext, rewrite: &Self::FixInput<'_>) -> Option<Fix> {
        match rewrite {
            OrOperatorRewrite::BooleanKeyword(operator_span) => Some(Fix {
                explanation: "Use the `or` operator".into(),
                replacements: vec![Replacement::new(*operator_span, "or".to_string())],
            }),
            OrOperatorRewrite::TryCatchNesting(nesting_edits) => Some(Fix {
                explanation: "Run the fallback with `try`/`catch`".into(),
                replacements: nesting_edits.clone(),
            }),
            OrOperatorRewrite::NoEquivalent => None,
        }
    }
}

pub static RULE: &dyn Rule = &ShellOrToTry;
