use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'input' or 'input -s' for password input.";

struct UseBuiltinRead;

impl DetectFix for UseBuiltinRead {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "read_to_input"
    }

    fn short_description(&self) -> &'static str {
        "`read` replaceable with `input`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/input.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_external_with_validation("read", |_, _, _| Some(NOTE))
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let arg_texts: Vec<&str> = fix_data.arg_texts(context).collect();
        let (repl, desc) = if arg_texts.iter().any(|s| *s == "-s" || *s == "--silent") {
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
        Some(Fix {
            explanation: desc.into(),
            replacements: vec![Replacement::new(fix_data.expr_span, repl)],
        })
    }
}

pub static RULE: &dyn Rule = &UseBuiltinRead;

#[cfg(test)]
mod tests {
    use super::RULE;

    #[test]
    fn converts_read_to_input() {
        let source = "^read";
        RULE.assert_fixed_contains(source, "input");
    }

    #[test]
    fn converts_read_silent_to_input_secure() {
        let source = "^read -s";
        RULE.assert_fixed_contains(source, "input -s");
    }
}
