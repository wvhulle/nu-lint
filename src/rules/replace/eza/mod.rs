use nu_protocol::ast::ExternalArgument;

use crate::{
    Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE: &str = "Use Nu's built-in 'ls' which returns structured data.";

fn build_fix(
    _cmd_text: &str,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let args_text: Vec<&str> = external_args_slices(args, context).collect();
    let replacement = if args_text.is_empty() {
        "ls".to_string()
    } else {
        format!("ls {}", args_text.join(" "))
    };
    Fix::with_explanation(
        NOTE.to_string(),
        vec![Replacement::new(expr_span, replacement)],
    )
}

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(context, "prefer_builtin_eza", "eza", NOTE, Some(build_fix))
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_eza",
        "Use Nu's built-in 'ls' instead of eza",
        check,
    )
}

#[cfg(test)]
mod tests {
    use super::rule;

    #[test]
    fn detects_eza_command() {
        let source = "^eza -la";
        rule().assert_replacement_contains(source, "ls");
    }
}
