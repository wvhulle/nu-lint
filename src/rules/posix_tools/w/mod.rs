use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'sys users' to get structured information about logged-in users. For \
                    system load, use 'sys host' or 'sys cpu'. Nu provides structured data instead \
                    of text.";

struct UseSysUsersInsteadOfW;

impl DetectFix for UseSysUsersInsteadOfW {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "use_sys_users_instead_of_w"
    }

    fn short_description(&self) -> &'static str {
        "Use Nu's 'sys users' command instead of 'w' for user information"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/sys_users.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_external_with_validation("w", |_, _, _| Some(NOTE))
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = "sys users";
        let description = "Use 'sys users' to get user information. For system load and uptime, \
                           use 'sys host' or 'sys cpu'. Nu provides structured data you can \
                           easily work with.";

        Some(Fix {
            explanation: description.into(),
            replacements: vec![Replacement {
                span: fix_data.expr_span.into(),
                replacement_text: replacement.into(),
            }],
        })
    }
}

pub static RULE: &dyn Rule = &UseSysUsersInsteadOfW;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
