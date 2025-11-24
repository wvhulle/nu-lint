use nu_protocol::ast::ExternalArgument;

use crate::{
    Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

fn build_fix(
    _cmd_text: &str,
    _builtin_cmd: &str,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    // Re-use grep's option parsing by a simple heuristic: prefer find for simplest case
    let pattern = external_args_slices(args, context)
        .next()
        .unwrap_or("pattern");
    let replacement = format!("find \"{}\"", pattern.trim_matches('"'));
    let description = "Use 'find' or 'lines | where' instead of rg. Nu is case-insensitive by default and works on structured data.";
    Fix {
        explanation: description.into(),
        replacements: vec![Replacement {
            span: expr_span,
            replacement_text: replacement.into(),
        }],
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(
        context,
        "prefer_builtin_rg",
        "rg",
        "find or where",
        "Use 'find' for simple text search or 'where $it =~ pattern' for regex filtering. Nushell's find/where operate on structured data and integrate with pipelines.",
        Some(build_fix),
    )
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_rg",
        "Use Nu's 'find' or 'where' instead of 'rg'",
        check,
    )
}

#[cfg(test)]
mod tests {
    use super::rule;

    #[test]
    fn detect_ripgrep() {
        rule().assert_detects(r#"^rg \"pattern\""#);
    }

    #[test]
    fn detect_ripgrep_with_file() {
        rule().assert_detects(r#"^rg \"error\" logs.txt"#);
    }

    #[test]
    fn ignore_nushell_alternatives() {
        rule().assert_ignores(r#"find \"pattern\""#);
        rule().assert_ignores(r#"lines | where $it =~ \"pattern\""#);
    }
}
