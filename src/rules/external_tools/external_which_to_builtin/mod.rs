use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'which' to find command locations.";

struct UseBuiltinWhich;

impl DetectFix for UseBuiltinWhich {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "external_which_to_builtin"
    }

    fn short_description(&self) -> &'static str {
        "External `which` replaceable with built-in"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/which.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        // which has a direct Nu builtin equivalent
        context.detect_external_with_validation("which", |_, _, _| Some(NOTE))
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let arg_texts: Vec<&str> = fix_data.arg_texts(context).collect();
        let repl = arg_texts
            .first()
            .map_or_else(|| "which".to_string(), |arg| format!("which {arg}"));
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
