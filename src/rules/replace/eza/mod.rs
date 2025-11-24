use crate::{Violation, alternatives::detect_external_commands, context::LintContext, rule::Rule};

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(
        context,
        "prefer_builtin_eza",
        "eza",
        "ls",
        "Use Nu's built-in 'ls' which returns structured data.",
        None,
    )
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_eza",
        "Use Nu's built-in 'ls' instead of eza",
        check,
    )
}

#[cfg(test)]
mod tests {
    use super::rule;

    #[test]
    fn detects_eza_command() {
        let source = "^eza -la";
        rule().assert_replacement_contains(source, "ls");
    }
}
