use std::sync::OnceLock;

use nu_lint_analysis::{
    LintLevel,
    ast::lexed_statement::all_lexed_statements,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::{ParseError, Span};
use regex::Regex;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;

const LAST_EXIT_CODE_VARIABLE: &str = "$?";

fn shell_variable_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"^\$(?:\{(?P<braced>[A-Z_][A-Z0-9_]*)\}|(?P<bare>[A-Z_][A-Z0-9_]*))(?P<suffix>[\w./:-]*)$")
            .unwrap()
    })
}

fn nu_env_expression(token_text: &str) -> Option<String> {
    if token_text == LAST_EXIT_CODE_VARIABLE {
        return Some("$env.LAST_EXIT_CODE".to_string());
    }
    let captures = shell_variable_pattern().captures(token_text)?;
    let variable_name = captures
        .name("braced")
        .or_else(|| captures.name("bare"))?
        .as_str();
    match captures.name("suffix").map_or("", |suffix| suffix.as_str()) {
        "" => Some(format!("$env.{variable_name}")),
        path_suffix => Some(format!("$\"($env.{variable_name}){path_suffix}\"")),
    }
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

fn contains_unresolved_variable_error(token_span: Span, context: &LintContext) -> bool {
    context.working_set.parse_errors.iter().any(|parse_error| {
        !is_type_check_error(parse_error) && token_span.contains_span(parse_error.span())
    })
}

struct EnvVariableRewrite {
    variable_span: Span,
    env_expression: String,
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
            .flat_map(|statement| &statement.statement_tokens)
            .filter(|token| contains_unresolved_variable_error(token.span, context))
            .filter_map(|token| {
                let env_expression = nu_env_expression(context.span_text(token.span))?;
                let detection =
                    Detection::from_global_span("Bash variable is not defined in Nu", token.span)
                        .with_primary_label("bash variable");
                Some((
                    detection,
                    EnvVariableRewrite {
                        variable_span: token.span,
                        env_expression,
                    },
                ))
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, rewrite: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix {
            explanation: "Read the variable from `$env`".into(),
            replacements: vec![Replacement::new(
                rewrite.variable_span,
                rewrite.env_expression.clone(),
            )],
        })
    }
}

pub static RULE: &dyn Rule = &ShellVariableToEnv;
