use nu_protocol::ast::ExternalArgument;

use crate::{
    Violation,
    alternatives::detect_external_commands,
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE: &str = "Use 'where' for filtering, 'select' for columns, or 'each' for row-by-row processing.";

fn build_fix(
    _cmd_text: &str,
    builtin_cmd: &str,
    _args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    _context: &LintContext,
) -> Fix {
    Fix::with_explanation(
        "Use Nushell pipeline primitives",
        vec![Replacement::new(expr_span, builtin_cmd.to_string())],
    )
}

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(
        context,
        "prefer_builtin_awk",
        "awk",
        "where | select | each",
        NOTE,
        Some(build_fix),
    )
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_awk",
        "Use Nushell pipelines (where/select/each) instead of awk",
        check,
    )
}

#[cfg(test)]
mod tests {
    use super::rule;

    #[test]
    fn converts_awk_to_nu_pipeline() {
        let source = "^awk";
        rule().assert_replacement_contains(source, "where | select | each");
        rule().assert_fix_explanation_contains(source, "pipeline");
    }
}
