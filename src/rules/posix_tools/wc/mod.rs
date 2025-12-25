use crate::{
    LintLevel,
    alternatives::{ExternalCmdFixData, detect_external_commands, external_args_slices},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'length' for item count or 'str length' for character count.";

struct UseBuiltinWc;

impl DetectFix for UseBuiltinWc {
    type FixInput = ExternalCmdFixData;

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

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
        detect_external_commands(context, "wc", NOTE)
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput) -> Option<Fix> {
        let (replacement, description) =
            if external_args_slices(&fix_data.args, context).any(|x| x == "-l") {
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
