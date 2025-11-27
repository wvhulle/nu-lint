use nu_protocol::ast::ExternalArgument;

use crate::{
    Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE: &str = "Use 'which' to find command locations.";

fn build_fix(
    _cmd_text: &str,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let args_text: Vec<&str> = external_args_slices(args, context).collect();
    let repl = args_text
        .first()
        .map_or_else(|| "which".to_string(), |cmd| format!("which {cmd}"));
    Fix::with_explanation(
        "Use built-in which",
        vec![Replacement::new(expr_span, repl)],
    )
}

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(context,
        "which",
        NOTE,
        Some(build_fix),
    )
}

pub const fn rule() -> Rule {
    Rule::new("prefer_builtin_which", "Prefer built-in 'which'", check)
        .with_doc_url("https://www.nushell.sh/commands/docs/which.html")
}

#[cfg(test)]
mod tests {
    use super::rule;

    #[test]
    fn converts_which_to_builtin_which() {
        let source = "^which python";
        rule().assert_replacement_contains(source, "which python");
    }
}
