use nu_lint_analysis::{
    LintLevel,
    ast::lexed_statement::{
        LexedStatement, Token, all_lexed_statements, is_pipe_token, token_source_bytes,
    },
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::Span;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;

const STDERR_REDIRECT_PREFIX: &[u8] = b"2>";
const MERGED_STREAMS_OPERATOR: &[u8] = b"2>&1";

enum RedirectRewrite {
    MergedStreamsBeforePipe(Span),
    TrailingMergedStreams(Span),
    StderrSpacedTarget(Span),
    StderrAttachedTarget(Span),
    NoEquivalent,
}

struct RedirectOccurrence {
    operator_span: Span,
    redirect_rewrite: RedirectRewrite,
}

fn merged_streams_rewrite(
    lexed_tokens: &[Token],
    operator_index: usize,
    lint_context: &LintContext,
) -> RedirectRewrite {
    let operator_span = lexed_tokens[operator_index].span;
    match lexed_tokens.get(operator_index + 1) {
        Some(next_token) if is_pipe_token(next_token, lint_context) => {
            RedirectRewrite::MergedStreamsBeforePipe(Span::new(
                operator_span.start,
                next_token.span.end,
            ))
        }
        None => RedirectRewrite::TrailingMergedStreams(Span::new(
            lexed_tokens[operator_index - 1].span.end,
            operator_span.end,
        )),
        Some(_) => RedirectRewrite::NoEquivalent,
    }
}

fn stderr_target_rewrite(operator_span: Span, redirect_target: &[u8]) -> RedirectRewrite {
    let prefix_span = Span::new(
        operator_span.start,
        operator_span.start + STDERR_REDIRECT_PREFIX.len(),
    );
    if redirect_target.is_empty() {
        RedirectRewrite::StderrSpacedTarget(prefix_span)
    } else {
        RedirectRewrite::StderrAttachedTarget(prefix_span)
    }
}

fn statement_redirects(
    lexed_statement: &LexedStatement,
    lint_context: &LintContext,
) -> Vec<RedirectOccurrence> {
    let lexed_tokens = &lexed_statement.statement_tokens;
    (1..lexed_tokens.len())
        .filter_map(|token_index| {
            let operator_span = lexed_tokens[token_index].span;
            let token_bytes = token_source_bytes(&lexed_tokens[token_index], lint_context);
            let redirect_rewrite = if token_bytes == MERGED_STREAMS_OPERATOR {
                merged_streams_rewrite(lexed_tokens, token_index, lint_context)
            } else {
                stderr_target_rewrite(
                    operator_span,
                    token_bytes.strip_prefix(STDERR_REDIRECT_PREFIX)?,
                )
            };
            Some(RedirectOccurrence {
                operator_span,
                redirect_rewrite,
            })
        })
        .collect()
}

fn redirect_fix(replaced_span: Span, nu_redirect_text: &str) -> Fix {
    Fix {
        explanation: "Use Nu redirection syntax".into(),
        replacements: vec![Replacement::new(
            replaced_span,
            nu_redirect_text.to_string(),
        )],
    }
}

struct ShellRedirectToNu;

impl DetectFix for ShellRedirectToNu {
    type FixInput<'a> = RedirectRewrite;

    fn id(&self) -> &'static str {
        "shell_redirect_to_nu"
    }

    fn short_description(&self) -> &'static str {
        "Bash stderr redirection replaceable with `err>` or `out+err>|`"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "The parser rejects `2>&1` and `2> file`, and reads `2>file` without a space as a \
             plain argument, so nothing gets redirected.",
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
            .flat_map(|lexed_statement| statement_redirects(lexed_statement, context))
            .map(|redirect_occurrence| {
                (
                    Detection::from_global_span(
                        "Bash redirection is not valid Nu syntax",
                        redirect_occurrence.operator_span,
                    )
                    .with_primary_label("bash redirection"),
                    redirect_occurrence.redirect_rewrite,
                )
            })
            .collect()
    }

    fn fix(
        &self,
        _lint_context: &LintContext,
        redirect_rewrite: &Self::FixInput<'_>,
    ) -> Option<Fix> {
        match redirect_rewrite {
            RedirectRewrite::MergedStreamsBeforePipe(replaced_span) => {
                Some(redirect_fix(*replaced_span, "out+err>|"))
            }
            RedirectRewrite::TrailingMergedStreams(replaced_span) => {
                Some(redirect_fix(*replaced_span, ""))
            }
            RedirectRewrite::StderrSpacedTarget(prefix_span) => {
                Some(redirect_fix(*prefix_span, "err>"))
            }
            RedirectRewrite::StderrAttachedTarget(prefix_span) => {
                Some(redirect_fix(*prefix_span, "err> "))
            }
            RedirectRewrite::NoEquivalent => None,
        }
    }
}

pub static RULE: &dyn Rule = &ShellRedirectToNu;
