use crate::{
    LintLevel,
    context::LintContext,
    external_commands::{ExternalCmdFixData, detect_external_commands},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'length' for item count or 'str length' for character count.";

struct UseBuiltinWc;

impl DetectFix for UseBuiltinWc {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "use_builtin_wc"
    }

    fn explanation(&self) -> &'static str {
        "Prefer 'length' over external wc"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/length.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        detect_external_commands(context, "wc", NOTE)
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let (replacement, description) = if fix_data.arg_strings.iter().copied().any(|x| x == "-l")
        {
            (
                "lines | length".to_string(),
                "Use 'lines | length' to count lines in a file".to_string(),
            )
        } else {
            (
                "length".to_string(),
                "Use 'length' for item count or 'str length' for character count".to_string(),
            )
        };
        Some(Fix::with_explanation(
            description,
            vec![Replacement::new(fix_data.expr_span, replacement)],
        ))
    }
}

pub static RULE: &dyn Rule = &UseBuiltinWc;

#[cfg(test)]
mod tests {
    use super::RULE;

    #[test]
    fn converts_wc_lines_to_lines_length() {
        let source = "^wc -l";
        RULE.assert_replacement_contains(source, "lines | length");
        RULE.assert_fix_explanation_contains(source, "count");
    }
}
