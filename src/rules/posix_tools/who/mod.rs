use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'sys users' to get structured information about logged-in users. Nu's sys \
                    users returns a table with user, terminal, and login_time fields.";

struct UseSysUsersInsteadOfWho;

impl DetectFix for UseSysUsersInsteadOfWho {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "use_sys_users_instead_of_who"
    }

    fn explanation(&self) -> &'static str {
        "Use Nu's 'sys users' command instead of 'who' for user information"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/sys_users.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_external_with_validation("who", |_, _, _| Some(NOTE))
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = "sys users";
        let description = "Use 'sys users' to get structured information about logged-in users. \
                           Returns a table with user, terminal, and login_time that you can \
                           filter and manipulate.";

        Some(Fix {
            explanation: description.into(),
            replacements: vec![Replacement {
                span: fix_data.expr_span.into(),
                replacement_text: replacement.into(),
            }],
        })
    }
}

pub static RULE: &dyn Rule = &UseSysUsersInsteadOfWho;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
