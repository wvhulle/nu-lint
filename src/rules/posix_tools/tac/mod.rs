use crate::{
    LintLevel,
    context::LintContext,
    external_commands::ExternalCmdFixData,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'open --raw | lines | reverse' to reverse file content in Nushell. \
                    Unlike tac which outputs text, Nu's pipeline returns a list of lines \
                    that can be further processed.";

struct UseBuiltinTac;

impl DetectFix for UseBuiltinTac {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "use_builtin_tac"
    }

    fn explanation(&self) -> &'static str {
        "Use Nu's 'open --raw | lines | reverse' instead of 'tac' for reversing file content"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/reverse.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.external_invocations("tac", NOTE)
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let filename = fix_data.arg_strings.iter().find(|s| !s.starts_with('-'));

        let replacement = filename.map_or_else(
            || "open --raw | lines | reverse".to_string(),
            |file| format!("open --raw {file} | lines | reverse"),
        );

        let description =
            "Use 'open --raw | lines | reverse' to reverse lines. Add '| str join \"\\n\"' \
             if you need text output instead of a list.";

        Some(Fix {
            explanation: description.into(),
            replacements: vec![Replacement {
                span: fix_data.expr_span.into(),
                replacement_text: replacement.into(),
            }],
        })
    }
}

pub static RULE: &dyn Rule = &UseBuiltinTac;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
