use crate::{
    LintLevel,
    context::LintContext,
    external_commands::{ExternalCmdFixData, detect_external_commands},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'select' to choose specific columns.";

struct UseBuiltinCut;

impl DetectFix for UseBuiltinCut {
    type FixInput = ExternalCmdFixData;

    fn id(&self) -> &'static str {
        "use_builtin_cut"
    }

    fn explanation(&self) -> &'static str {
        "Use 'select' instead of external cut"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/select.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
        detect_external_commands(context, "cut", NOTE)
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput) -> Option<Fix> {
        Some(Fix::with_explanation(
            "Use 'select' for columns",
            vec![Replacement::new(fix_data.expr_span, "select".to_string())],
        ))
    }
}

pub static RULE: &dyn Rule = &UseBuiltinCut;

#[cfg(test)]
mod tests {
    use super::RULE;

    #[test]
    fn converts_cut_to_select() {
        let source = "^cut";
        RULE.assert_replacement_contains(source, "select");
        RULE.assert_fix_explanation_contains(source, "columns");
    }
}
