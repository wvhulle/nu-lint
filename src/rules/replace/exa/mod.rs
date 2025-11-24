use crate::{Violation, alternatives::detect_external_commands, context::LintContext, rule::Rule};

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(
        context,
        "prefer_builtin_exa",
        "exa",
        "ls",
        "Use Nu's built-in 'ls' which returns structured data.",
        None,
    )
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_exa",
        "Use Nu's built-in 'ls' instead of exa",
        check,
    )
}

#[cfg(test)]
mod tests {
    use super::rule;

    #[test]
    fn detects_exa_command() {
        let source = "^exa";
        rule().assert_replacement_contains(source, "ls");
    }
}
