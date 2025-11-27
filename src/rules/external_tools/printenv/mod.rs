use nu_protocol::ast::ExternalArgument;

use crate::{
    Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE: &str = "Use '$env' to access environment variables or 'env' to view all.";

fn build_fix(
    _cmd_text: &str,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let args_text: Vec<&str> = external_args_slices(args, context).collect();

    let (replacement, description) = if args_text.is_empty() {
        (
            "$env".to_string(),
            "Use '$env' to access all environment variables as a record".to_string(),
        )
    } else {
        (
            format!("$env.{}", args_text[0]),
            format!(
                "Use '$env.{}' to access environment variable directly",
                args_text[0]
            ),
        )
    };

    Fix::with_explanation(description, vec![Replacement::new(expr_span, replacement)])
}

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(context,
        "printenv",
        NOTE,
        Some(build_fix),
    )
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_printenv",
        "Prefer $env over printenv",
        check,
    )
    .with_doc_url("https://www.nushell.sh/book/environment.html")
}

#[cfg(test)]
mod tests {
    use super::rule;

    #[test]
    fn converts_printenv_to_env_variable_access() {
        let source = "^printenv HOME";
        rule().assert_replacement_contains(source, "$env.HOME");
        rule().assert_fix_explanation_contains(source, "directly");
    }
}
