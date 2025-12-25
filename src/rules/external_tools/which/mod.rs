use crate::{
    LintLevel,
    alternatives::{ExternalCmdFixData, detect_external_commands, external_args_slices},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'which' to find command locations.";

struct UseBuiltinWhich;

impl DetectFix for UseBuiltinWhich {
    type FixInput = ExternalCmdFixData;

    fn id(&self) -> &'static str {
        "use_builtin_which"
    }

    fn explanation(&self) -> &'static str {
        "Prefer built-in 'which'"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/which.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
        detect_external_commands(context, "which", NOTE)
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput) -> Option<Fix> {
        let args_text: Vec<&str> = external_args_slices(&fix_data.args, context).collect();
        let repl = args_text
            .first()
            .map_or_else(|| "which".to_string(), |cmd| format!("which {cmd}"));
        Some(Fix::with_explanation(
            "Use built-in which",
            vec![Replacement::new(fix_data.expr_span, repl)],
        ))
    }
}

pub static RULE: &dyn Rule = &UseBuiltinWhich;

#[cfg(test)]
mod tests {
    use super::RULE;

    #[test]
    fn converts_which_to_builtin_which() {
        let source = "^which python";
        RULE.assert_replacement_contains(source, "which python");
    }
}
