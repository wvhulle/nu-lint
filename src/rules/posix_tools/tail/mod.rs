use crate::{
    LintLevel,
    context::LintContext,
    external_commands::{ExternalCmdFixData, detect_external_commands},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'last N' to get the last N items";

struct UseBuiltinTail;

impl DetectFix for UseBuiltinTail {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "use_builtin_tail"
    }

    fn explanation(&self) -> &'static str {
        "Use Nu's 'last' command instead of 'tail' for cleaner syntax"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/last.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        detect_external_commands(context, "tail", NOTE)
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = fix_data
            .arg_strings
            .iter()
            .copied()
            .find(|a| a.starts_with('-') && a.len() > 1)
            .map_or_else(
                || "last 10".to_string(),
                |num_arg| {
                    let num = &num_arg[1..];
                    format!("last {num}")
                },
            );

        let description = "Use 'last' with cleaner syntax: 'last N' instead of 'tail -N'";

        Some(Fix {
            explanation: description.into(),
            replacements: vec![Replacement {
                span: fix_data.expr_span.into(),
                replacement_text: replacement.into(),
            }],
        })
    }
}

pub static RULE: &dyn Rule = &UseBuiltinTail;

#[cfg(test)]
mod tests;
