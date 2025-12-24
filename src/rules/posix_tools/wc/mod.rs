use nu_protocol::ast::ExternalArgument;

use crate::{
    LintLevel, Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE: &str = "Use 'length' for item count or 'str length' for character count.";

fn build_fix(
    _cmd_text: &str,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let (replacement, description) = if external_args_slices(args, context).any(|x| x == "-l") {
        (
            "lines | length".to_string(),
            "Use 'lines | length' to count lines in a file".to_string(),
        )
    } else {
        (
            "length".to_string(),
            "Use 'length' for item count or 'str length' for character count".to_string(),
        )
    };
    Fix::with_explanation(description, vec![Replacement::new(expr_span, replacement)])
}

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(context, "wc", NOTE, Some(build_fix))
}

pub const RULE: Rule = Rule::new(
    "use_builtin_wc",
    "Prefer 'length' over external wc",
    check,
    LintLevel::Warning,
)
.with_auto_fix()
.with_doc_url("https://www.nushell.sh/commands/docs/length.html");

#[cfg(test)]
mod tests {
    use super::RULE;

    #[test]
    fn converts_wc_lines_to_lines_length() {
        let source = "^wc -l";
        RULE.assert_replacement_contains(source, "lines | length");
        RULE.assert_fix_explanation_contains(source, "count");
    }
}
