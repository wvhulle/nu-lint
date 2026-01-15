use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct UseBuiltinCut;

impl DetectFix for UseBuiltinCut {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "use_builtin_cut"
    }

    fn explanation(&self) -> &'static str {
        "Use 'select' instead of external cut"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/select.html")
    }

    fn level(&self) -> Option<LintLevel> {
            Some(LintLevel::Warning)
        }
    

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        // Don't detect cut at all - select is for structured data, cut is for text processing
        // They operate in different domains and translation is unreliable
        context.detect_external_with_validation("cut", |_, _, _| None)
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix::with_explanation(
            "Use 'select' for columns",
            vec![Replacement::new(fix_data.expr_span, "select".to_string())],
        ))
    }
}

pub static RULE: &dyn Rule = &UseBuiltinCut;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;

