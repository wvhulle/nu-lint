use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'sys host | get uptime' to get system uptime. Nu's sys host returns \
                    structured information about the host system including uptime as a duration \
                    value.";

struct UseSysHostInsteadOfUptime;

impl DetectFix for UseSysHostInsteadOfUptime {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "use_sys_host_instead_of_uptime"
    }

    fn explanation(&self) -> &'static str {
        "Use Nu's 'sys host | get uptime' command instead of 'uptime'"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/sys_host.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_external_with_validation("uptime", |_, _| Some(NOTE))
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = "sys host | get uptime";
        let description = "Use 'sys host | get uptime' to get system uptime as a duration value. \
                           This is more convenient than parsing uptime's text output.";

        Some(Fix {
            explanation: description.into(),
            replacements: vec![Replacement {
                span: fix_data.expr_span.into(),
                replacement_text: replacement.into(),
            }],
        })
    }
}

pub static RULE: &dyn Rule = &UseSysHostInsteadOfUptime;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
