use crate::{
    LintLevel,
    alternatives::{ExternalCmdFixData, detect_external_commands, external_args_slices},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'last N' to get the last N items";

struct UseBuiltinTail;

impl DetectFix for UseBuiltinTail {
    type FixInput = ExternalCmdFixData;

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

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
        detect_external_commands(context, "tail", NOTE)
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput) -> Option<Fix> {
        let replacement = external_args_slices(&fix_data.args, context)
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
