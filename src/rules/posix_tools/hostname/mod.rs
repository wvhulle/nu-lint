use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'sys host | get hostname' to get the system hostname. Nu's sys host \
                    returns structured information about the host system.";

struct UseSysHostInsteadOfHostname;

impl DetectFix for UseSysHostInsteadOfHostname {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "hostname_to_sys_host"
    }

    fn short_description(&self) -> &'static str {
        "`hostname` replaceable with `sys host`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/sys_host.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_external_with_validation("hostname", |_, _, _| Some(NOTE))
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = "sys host | get hostname";
        let description =
            "Use 'sys host | get hostname' to get the system hostname as a string value.";

        Some(Fix {
            explanation: description.into(),
            replacements: vec![Replacement {
                span: fix_data.expr_span.into(),
                replacement_text: replacement.into(),
            }],
        })
    }
}

pub static RULE: &dyn Rule = &UseSysHostInsteadOfHostname;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
