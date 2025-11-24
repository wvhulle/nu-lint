use nu_protocol::ast::ExternalArgument;

use crate::{
    Violation,
    alternatives::detect_external_commands,
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE: &str = "Use 'date now' or parse dates with 'into datetime'.";

fn build_fix(
    _cmd_text: &str,
    builtin_cmd: &str,
    _args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    _context: &LintContext,
) -> Fix {
    Fix::with_explanation(
        "Use 'date now' which returns a datetime object",
        vec![Replacement::new(expr_span, builtin_cmd.to_string())],
    )
}

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(
        context,
        "prefer_builtin_date",
        "date",
        "date now",
        NOTE,
        Some(build_fix),
    )
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_date",
        "Use 'date now' instead of external date",
        check,
    )
}

#[cfg(test)]
mod tests {
    use super::rule;

    #[test]
    fn converts_date_command_to_date_now() {
        let source = "^date";
        rule().assert_replacement_contains(source, "date now");
        rule().assert_fix_explanation_contains(source, "datetime");
    }
}
