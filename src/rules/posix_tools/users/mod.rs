use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'sys users | get user' to get a list of logged-in usernames. Nu's sys \
                    users returns structured data that's easier to manipulate.";

struct UseSysUsersInsteadOfUsers;

impl DetectFix for UseSysUsersInsteadOfUsers {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "use_sys_users_instead_of_users"
    }

    fn explanation(&self) -> &'static str {
        "Use Nu's 'sys users | get user' command instead of 'users'"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/sys_users.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_external_with_validation("users", |_, _| Some(NOTE))
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = "sys users | get user";
        let description = "Use 'sys users | get user' to get a list of logged-in usernames. This \
                           gives you structured data instead of space-separated text.";

        Some(Fix {
            explanation: description.into(),
            replacements: vec![Replacement {
                span: fix_data.expr_span.into(),
                replacement_text: replacement.into(),
            }],
        })
    }
}

pub static RULE: &dyn Rule = &UseSysUsersInsteadOfUsers;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
