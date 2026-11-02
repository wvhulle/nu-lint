use nu_lint_analysis::{
    LintLevel,
    ast::lexed_statement::{LexedStatement, all_lexed_statements, token_text_equals},
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

enum AndOperatorRewrite {
    SeparateCommands(Span),
    BooleanKeyword(Span),
    NoEquivalent,
}

fn statement_and_operators(
    statement: &LexedStatement,
    context: &LintContext,
) -> Vec<(Detection, AndOperatorRewrite)> {
    let tokens = &statement.statement_tokens;
    let operand_context = classify_operand_context(statement, context);
    tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| token_text_equals(token, context, b"&&"))
        .map(|(token_index, token)| {
            let rewrite = match operand_context {
                OperandContext::CommandChain => {
                    let chain_start = token_index
                        .checked_sub(1)
                        .map_or(token.span.start, |previous_index| {
                            tokens[previous_index].span.end
                        });
                    AndOperatorRewrite::SeparateCommands(Span::new(chain_start, token.span.end))
                }
                OperandContext::BooleanExpression => AndOperatorRewrite::BooleanKeyword(token.span),
                OperandContext::BashTest => AndOperatorRewrite::NoEquivalent,
            };
            let detection =
                Detection::from_global_span("The `&&` operator does not exist in Nu", token.span)
                    .with_primary_label("bash operator");
            (detection, rewrite)
        })
        .collect()
}

struct ShellAndToSemicolon;

impl DetectFix for ShellAndToSemicolon {
    type FixInput<'a> = AndOperatorRewrite;

    fn id(&self) -> &'static str {
        "shell_and_to_semicolon"
    }

    fn short_description(&self) -> &'static str {
        "Bash `&&` replaceable with `;` or `and`"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "A failing command stops the statements after it, so `;` keeps the short-circuit \
             behaviour of `&&`. A bash test such as `[ -f x ] && y` gets no fix, because `[ -f x \
             ]` is a list here and `;` would run `y` unconditionally.",
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
            .flat_map(|statement| statement_and_operators(statement, context))
            .collect()
    }

    fn fix(&self, _context: &LintContext, rewrite: &Self::FixInput<'_>) -> Option<Fix> {
        match rewrite {
            AndOperatorRewrite::SeparateCommands(operator_span) => Some(Fix {
                explanation: "Separate the commands with `;`".into(),
                replacements: vec![Replacement::new(*operator_span, ";".to_string())],
            }),
            AndOperatorRewrite::BooleanKeyword(operator_span) => Some(Fix {
                explanation: "Use the `and` operator".into(),
                replacements: vec![Replacement::new(*operator_span, "and".to_string())],
            }),
            AndOperatorRewrite::NoEquivalent => None,
        }
    }
}

pub static RULE: &dyn Rule = &ShellAndToSemicolon;
