use crate::{
    LintLevel,
    alternatives::{ExternalCmdFixData, detect_external_commands, external_args_slices},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'input' or 'input -s' for password input.";

struct UseBuiltinRead;

impl DetectFix for UseBuiltinRead {
    type FixInput = ExternalCmdFixData;

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

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
        detect_external_commands(context, "read", NOTE)
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput) -> Option<Fix> {
        let args_text: Vec<&str> = external_args_slices(&fix_data.args, context).collect();
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
