use crate::{
    LintLevel,
    context::LintContext,
    external_commands::{ExternalCmdFixData},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'input' or 'input -s' for password input.";

struct UseBuiltinRead;

impl DetectFix for UseBuiltinRead {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "use_builtin_read"
    }

    fn explanation(&self) -> &'static str {
        "Prefer 'input' over 'read'"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/input.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.external_invocations("read", NOTE)
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let args_text: Vec<&str> = fix_data.arg_strings.clone();
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
        Some(Fix::with_explanation(
            desc,
            vec![Replacement::new(fix_data.expr_span, repl)],
        ))
    }
}

pub static RULE: &dyn Rule = &UseBuiltinRead;

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
