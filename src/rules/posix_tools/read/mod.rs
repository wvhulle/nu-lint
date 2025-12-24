use nu_protocol::ast::ExternalArgument;

use crate::{
    LintLevel, Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE: &str = "Use 'input' or 'input -s' for password input.";

fn build_fix(
    _cmd_text: &str,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let args_text: Vec<&str> = external_args_slices(args, context).collect();
    let (repl, desc) = if args_text.iter().any(|&s| s == "-s" || s == "--silent") {
        (
            "input -s".to_string(),
            "Use 'input -s' for secure password input (hidden)".to_string(),
        )
    } else {
        (
            "input".to_string(),
            "Use 'input' to read user input".to_string(),
        )
    };
    Fix::with_explanation(desc, vec![Replacement::new(expr_span, repl)])
}

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(context, "read", NOTE, Some(build_fix))
}

pub const RULE: Rule = Rule::new(
    "use_builtin_read",
    "Prefer 'input' over 'read'",
    check,
    LintLevel::Warning,
)
.with_auto_fix()
.with_doc_url("https://www.nushell.sh/commands/docs/input.html");

#[cfg(test)]
mod tests {
    use super::RULE;

    #[test]
    fn converts_read_to_input() {
        let source = "^read";
        RULE.assert_replacement_contains(source, "input");
    }

    #[test]
    fn converts_read_silent_to_input_secure() {
        let source = "^read -s";
        RULE.assert_replacement_contains(source, "input -s");
        RULE.assert_fix_explanation_contains(source, "password");
    }
}
