use nu_lint_analysis::{
    LintLevel,
    ast::lexed_statement::{LexedStatement, Token, all_lexed_statements, is_pipe_token},
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
    ReplaceText {
        replaced_span: Span,
        nu_redirect_text: String,
    },
    NoEquivalent,
}

fn merged_streams_rewrite(
    tokens: &[Token],
    operator_index: usize,
    context: &LintContext,
) -> RedirectRewrite {
    let operator_span = tokens[operator_index].span;
    match tokens.get(operator_index + 1) {
        Some(next_token) if is_pipe_token(next_token, context) => RedirectRewrite::ReplaceText {
            replaced_span: Span::new(operator_span.start, next_token.span.end),
            nu_redirect_text: "out+err>|".to_string(),
        },
        None => RedirectRewrite::ReplaceText {
            replaced_span: Span::new(tokens[operator_index - 1].span.end, operator_span.end),
            nu_redirect_text: String::new(),
        },
        Some(_) => RedirectRewrite::NoEquivalent,
    }
}

fn stderr_target_rewrite(operator_span: Span, redirect_target: &[u8]) -> RedirectRewrite {
    let nu_redirect_text = if redirect_target.is_empty() {
        "err>".to_string()
    } else {
        format!("err> {}", String::from_utf8_lossy(redirect_target))
    };
    RedirectRewrite::ReplaceText {
        replaced_span: operator_span,
        nu_redirect_text,
    }
}

fn statement_redirects(
    statement: &LexedStatement,
    context: &LintContext,
) -> Vec<(Detection, RedirectRewrite)> {
    let tokens = &statement.statement_tokens;
    tokens
        .iter()
        .enumerate()
        .skip(1)
        .filter_map(|(token_index, token)| {
            let token_contents = context.working_set.get_span_contents(token.span);
            let rewrite = if token_contents == MERGED_STREAMS_OPERATOR {
                merged_streams_rewrite(tokens, token_index, context)
            } else {
                stderr_target_rewrite(
                    token.span,
                    token_contents.strip_prefix(STDERR_REDIRECT_PREFIX)?,
                )
            };
            let detection =
                Detection::from_global_span("Bash redirection is not valid Nu syntax", token.span)
                    .with_primary_label("bash redirection");
            Some((detection, rewrite))
        })
        .collect()
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
            .flat_map(|statement| statement_redirects(statement, context))
            .collect()
    }

    fn fix(&self, _context: &LintContext, rewrite: &Self::FixInput<'_>) -> Option<Fix> {
        match rewrite {
            RedirectRewrite::ReplaceText {
                replaced_span,
                nu_redirect_text,
            } => Some(Fix {
                explanation: "Use Nu redirection syntax".into(),
                replacements: vec![Replacement::new(*replaced_span, nu_redirect_text.clone())],
            }),
            RedirectRewrite::NoEquivalent => None,
        }
    }
}

pub static RULE: &dyn Rule = &ShellRedirectToNu;
