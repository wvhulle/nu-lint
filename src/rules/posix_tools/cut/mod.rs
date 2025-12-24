use nu_protocol::ast::ExternalArgument;

use crate::{
    LintLevel, Violation,
    alternatives::detect_external_commands,
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE: &str = "Use 'select' to choose specific columns.";

fn build_fix(
    _cmd_text: &str,
    _args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    _context: &LintContext,
) -> Fix {
    Fix::with_explanation(
        "Use 'select' for columns",
        vec![Replacement::new(expr_span, "select".to_string())],
    )
}

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(context, "cut", NOTE, Some(build_fix))
}

pub const RULE: Rule = Rule::new(
    "use_builtin_cut",
    "Use 'select' instead of external cut",
    check,
    LintLevel::Warning,
)
.with_doc_url("https://www.nushell.sh/commands/docs/select.html");

#[cfg(test)]
mod tests {
    use super::RULE;

    #[test]
    fn converts_cut_to_select() {
        let source = "^cut";
        RULE.assert_replacement_contains(source, "select");
        RULE.assert_fix_explanation_contains(source, "columns");
    }
}
