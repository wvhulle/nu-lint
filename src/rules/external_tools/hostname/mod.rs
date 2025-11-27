use nu_protocol::ast::ExternalArgument;

use crate::{
    Violation,
    alternatives::detect_external_commands,
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE: &str = "Use '(sys host).hostname' to get hostname, or 'sys host' for detailed info.";

fn build_fix(
    _cmd_text: &str,
    _args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    _context: &LintContext,
) -> Fix {
    Fix::with_explanation(
        "Use system info from 'sys host'",
        vec![Replacement::new(
            expr_span,
            "(sys host).hostname".to_string(),
        )],
    )
}

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(
        context,
        "prefer_builtin_hostname",
        "hostname",
        NOTE,
        Some(build_fix),
    )
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_hostname",
        "Use '(sys host).hostname' instead of external hostname",
        check,
    )
    .with_doc_url("https://www.nushell.sh/commands/docs/sys_host.html")
}

#[cfg(test)]
mod tests {
    use super::rule;

    #[test]
    fn converts_hostname_to_sys_host() {
        let source = "^hostname";
        rule().assert_replacement_contains(source, "(sys host).hostname");
        rule().assert_fix_explanation_contains(source, "sys");
    }
}
