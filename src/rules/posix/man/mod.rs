use nu_protocol::ast::{Expr, ExternalArgument};

use crate::{
    Violation,
    alternatives::external_args_slices,
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE: &str =
    "Use 'help <command>' for Nushell builtins or 'help commands' to list all commands.";

fn is_nushell_builtin(cmd_name: &str, context: &LintContext) -> bool {
    context.working_set.find_decl(cmd_name.as_bytes()).is_some()
}

fn build_fix(
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
    context.collect_rule_violations(|expr, ctx| {
        let Expr::ExternalCall(head, args) = &expr.expr else {
            return vec![];
        };

        let cmd_text = &ctx.source[head.span.start..head.span.end];
        if cmd_text != "man" {
            return vec![];
        }

        let args_text: Vec<&str> = external_args_slices(args, ctx).collect();
        let Some(first_arg) = args_text.first() else {
            return vec![];
        };

        if !is_nushell_builtin(first_arg, ctx) {
            return vec![];
        }

        let message = format!(
            "Consider using 'help {first_arg}' instead of 'man {first_arg}' for Nushell builtins"
        );
        let suggestion = format!(
            "'{first_arg}' is a Nushell builtin command. Use 'help {first_arg}' to see its \
             documentation instead of the external 'man' command.\n\nNote: {NOTE}"
        );

        let violation = Violation::new("prefer_builtin_man", message, expr.span)
            .with_help(suggestion)
            .with_fix(build_fix(args, expr.span, ctx));

        vec![violation]
    })
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_man",
        "Prefer 'help' over 'man' for Nushell builtins",
        check,
    )
}

#[cfg(test)]
mod tests {
    use super::rule;

    #[test]
    fn suggests_help_for_nushell_builtin() {
        let source = "^man ls";
        rule().assert_replacement_contains(source, "help ls");
    }

    #[test]
    fn suggests_help_for_each_builtin() {
        let source = "^man each";
        rule().assert_replacement_contains(source, "help each");
    }

    #[test]
    fn no_warning_for_external_only_command() {
        let source = "^man kubectl";
        rule().assert_ignores(source);
    }

    #[test]
    fn no_warning_for_git() {
        let source = "^man git";
        rule().assert_ignores(source);
    }

    #[test]
    fn no_warning_for_man_without_args() {
        let source = "^man";
        rule().assert_ignores(source);
    }
}
