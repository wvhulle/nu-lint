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

struct AndOperatorOccurrence {
    operator_span: Span,
    operator_rewrite: AndOperatorRewrite,
}

fn statement_and_operators(
    lexed_statement: &LexedStatement,
    lint_context: &LintContext,
) -> Vec<AndOperatorOccurrence> {
    let lexed_tokens = &lexed_statement.statement_tokens;
    let operand_context = classify_operand_context(lexed_statement, lint_context);
    (0..lexed_tokens.len())
        .filter(|&token_index| token_text_equals(&lexed_tokens[token_index], lint_context, b"&&"))
        .map(|token_index| {
            let operator_span = lexed_tokens[token_index].span;
            let operator_rewrite = match operand_context {
                OperandContext::CommandChain => {
                    let chain_start = token_index
                        .checked_sub(1)
                        .map_or(operator_span.start, |previous_index| {
                            lexed_tokens[previous_index].span.end
                        });
                    AndOperatorRewrite::SeparateCommands(Span::new(chain_start, operator_span.end))
                }
                OperandContext::BooleanExpression => {
                    AndOperatorRewrite::BooleanKeyword(operator_span)
                }
                OperandContext::UnclearOperands => AndOperatorRewrite::NoEquivalent,
            };
            AndOperatorOccurrence {
                operator_span,
                operator_rewrite,
            }
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
             behaviour of `&&`. There is no fix when the operands could be either commands or \
             booleans. For example `[ -f x ]` is a list here, and `;` would run what follows \
             unconditionally.",
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
            .flat_map(|lexed_statement| statement_and_operators(lexed_statement, context))
            .map(|operator_occurrence| {
                (
                    Detection::from_global_span(
                        "The `&&` operator does not exist in Nu",
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
