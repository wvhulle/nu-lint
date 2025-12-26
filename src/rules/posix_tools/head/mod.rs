use crate::{
    LintLevel,
    context::LintContext,
    external_commands::ExternalCmdFixData,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'first N' to get the first N items";

struct UseBuiltinHead;

impl DetectFix for UseBuiltinHead {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "use_builtin_head"
    }

    fn explanation(&self) -> &'static str {
        "Use Nu's 'first' command instead of 'head' for cleaner syntax"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/first.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.external_invocations("head", NOTE)
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = fix_data
            .arg_strings
            .iter()
            .copied()
            .find(|a| a.starts_with('-') && a.len() > 1)
            .map_or_else(
                || "first 10".to_string(),
                |num_arg| {
                    let num = &num_arg[1..];
                    format!("first {num}")
                },
            );

        let description = "Use 'first' with cleaner syntax: 'first N' instead of 'head -N'";

        Some(Fix {
            explanation: description.into(),
            replacements: vec![Replacement {
                span: fix_data.expr_span.into(),
                replacement_text: replacement.into(),
            }],
        })
    }
}

pub static RULE: &dyn Rule = &UseBuiltinHead;

#[cfg(test)]
mod tests;
