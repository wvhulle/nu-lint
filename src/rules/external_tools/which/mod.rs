use crate::{
    LintLevel,
    context::LintContext,
    external_commands::ExternalCmdFixData,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'which' to find command locations.";

struct UseBuiltinWhich;

impl DetectFix for UseBuiltinWhich {
    type FixInput<'a> = ExternalCmdFixData<'a>;

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

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.external_invocations("which", NOTE)
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let args_text: Vec<&str> = fix_data.arg_strings.clone();
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
        RULE.assert_fixed_contains(source, "which python");
    }
}
