use nu_protocol::ast::ExternalArgument;

use crate::{
    Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE: &str = "Use 'help <command>' or 'help commands' to list all commands.";

fn build_fix(
    _cmd_text: &str,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let args_text: Vec<&str> = external_args_slices(args, context).collect();
    let repl = args_text
        .first()
        .map_or_else(|| "help commands".to_string(), |cmd| format!("help {cmd}"));
    Fix::with_explanation(
        "Use help for Nushell commands",
        vec![Replacement::new(expr_span, repl)],
    )
}

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(context, "prefer_builtin_man", "man", NOTE, Some(build_fix))
}

pub const fn rule() -> Rule {
    Rule::new("prefer_builtin_man", "Prefer 'help' over 'man'", check)
}

#[cfg(test)]
mod tests {
    use super::rule;

    #[test]
    fn converts_man_to_help() {
        let source = "^man ls";
        rule().assert_replacement_contains(source, "help ls");
    }
}
