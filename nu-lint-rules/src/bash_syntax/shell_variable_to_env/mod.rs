use nu_lint_analysis::{
    LintLevel,
    ast::lexed_statement::{Token, all_lexed_statements, token_source_bytes},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::{ParseError, Span};

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;

const LAST_EXIT_CODE_VARIABLE: &[u8] = b"$?";

enum EnvVariableRewrite {
    LastExitCode {
        variable_span: Span,
    },
    WholeVariable {
        opening_span: Span,
        closing_span: Span,
    },
    InterpolatedPath {
        opening_span: Span,
        closing_span: Span,
        path_end: usize,
    },
}

struct EnvVariableOccurrence {
    variable_span: Span,
    variable_rewrite: EnvVariableRewrite,
}

const fn is_env_name_byte(name_byte: u8) -> bool {
    matches!(name_byte, b'A'..=b'Z' | b'0'..=b'9' | b'_')
}

const fn is_path_suffix_byte(suffix_byte: u8) -> bool {
    suffix_byte.is_ascii_alphanumeric() || matches!(suffix_byte, b'_' | b'.' | b'/' | b':' | b'-')
}

fn parse_shell_variable(variable_span: Span, token_bytes: &[u8]) -> Option<EnvVariableRewrite> {
    if token_bytes == LAST_EXIT_CODE_VARIABLE {
        return Some(EnvVariableRewrite::LastExitCode { variable_span });
    }
    let after_sigil = token_bytes.strip_prefix(b"$")?;
    let is_braced = after_sigil.first() == Some(&b'{');
    let name_offset = if is_braced { 2 } else { 1 };
    let name_bytes = token_bytes.get(name_offset..)?;
    let name_width = name_bytes
        .iter()
        .take_while(|name_byte| is_env_name_byte(**name_byte))
        .count();
    if name_width == 0 || name_bytes[0].is_ascii_digit() {
        return None;
    }
    let name_end = name_offset + name_width;
    let closing_end = if is_braced {
        (token_bytes.get(name_end) == Some(&b'}')).then_some(name_end + 1)?
    } else {
        name_end
    };
    let path_suffix = &token_bytes[closing_end..];
    if !path_suffix.iter().copied().all(is_path_suffix_byte) {
        return None;
    }

    let token_start = variable_span.start;
    let opening_span = Span::new(token_start, token_start + name_offset);
    let closing_span = Span::new(token_start + name_end, token_start + closing_end);
    Some(if path_suffix.is_empty() {
        EnvVariableRewrite::WholeVariable {
            opening_span,
            closing_span,
        }
    } else {
        EnvVariableRewrite::InterpolatedPath {
            opening_span,
            closing_span,
            path_end: variable_span.end,
        }
    })
}

const fn is_type_check_error(parse_error: &ParseError) -> bool {
    matches!(
        parse_error,
        ParseError::Mismatch(..)
            | ParseError::TypeMismatch(..)
            | ParseError::TypeMismatchHelp(..)
            | ParseError::InputMismatch(..)
            | ParseError::OutputMismatch(..)
    )
}

fn contains_unresolved_variable_error(token_span: Span, lint_context: &LintContext) -> bool {
    lint_context
        .working_set
        .parse_errors
        .iter()
        .any(|parse_error| {
            !is_type_check_error(parse_error) && token_span.contains_span(parse_error.span())
        })
}

fn env_variable_occurrence(
    lexed_token: &Token,
    lint_context: &LintContext,
) -> Option<EnvVariableOccurrence> {
    contains_unresolved_variable_error(lexed_token.span, lint_context)
        .then(|| {
            parse_shell_variable(
                lexed_token.span,
                token_source_bytes(lexed_token, lint_context),
            )
        })
        .flatten()
        .map(|variable_rewrite| EnvVariableOccurrence {
            variable_span: lexed_token.span,
            variable_rewrite,
        })
}

struct ShellVariableToEnv;

impl DetectFix for ShellVariableToEnv {
    type FixInput<'a> = EnvVariableRewrite;

    fn id(&self) -> &'static str {
        "shell_variable_to_env"
    }

    fn short_description(&self) -> &'static str {
        "Bash variable replaceable with `$env`"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Environment variables live under `$env`. This rule covers uppercase names and `$?`, \
             and only where the parser fails to resolve the variable, so constants such as `const \
             MAX = 1` are left alone.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/environment.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Error
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        all_lexed_statements(context)
            .iter()
            .flat_map(|lexed_statement| &lexed_statement.statement_tokens)
            .filter_map(|lexed_token| env_variable_occurrence(lexed_token, context))
            .map(|variable_occurrence| {
                (
                    Detection::from_global_span(
                        "Bash variable is not defined in Nu",
                        variable_occurrence.variable_span,
                    )
                    .with_primary_label("bash variable"),
                    variable_occurrence.variable_rewrite,
                )
            })
            .collect()
    }

    fn fix(
        &self,
        _lint_context: &LintContext,
        variable_rewrite: &Self::FixInput<'_>,
    ) -> Option<Fix> {
        let replacements = match variable_rewrite {
            EnvVariableRewrite::LastExitCode { variable_span } => vec![Replacement::new(
                *variable_span,
                "$env.LAST_EXIT_CODE".to_string(),
            )],
            EnvVariableRewrite::WholeVariable {
                opening_span,
                closing_span,
            } => vec![
                Replacement::new(*opening_span, "$env.".to_string()),
                Replacement::new(*closing_span, String::new()),
            ],
            EnvVariableRewrite::InterpolatedPath {
                opening_span,
                closing_span,
                path_end,
            } => vec![
                Replacement::new(*opening_span, "$\"($env.".to_string()),
                Replacement::new(*closing_span, ")".to_string()),
                Replacement::new(Span::new(*path_end, *path_end), "\"".to_string()),
            ],
        };
        Some(Fix {
            explanation: "Read the variable from `$env`".into(),
            replacements,
        })
    }
}

pub static RULE: &dyn Rule = &ShellVariableToEnv;
