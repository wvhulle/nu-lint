use nu_lint_analysis::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, DiagnosticTag, Fix},
};

use super::deprecated_usage::{CommandRename, MigratedDeprecation, command_renames};

pub const MIGRATIONS: &[MigratedDeprecation] =
    &[MigratedDeprecation::renamed_command("filter", "where")];

struct FilterToWhere;

impl DetectFix for FilterToWhere {
    type FixInput<'a> = CommandRename;

    fn id(&self) -> &'static str {
        "filter_to_where"
    }

    fn short_description(&self) -> &'static str {
        "Deprecated `filter` replaceable with `where`"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "`filter` was deprecated in Nushell 0.105.0. Since that release `where` accepts a \
             closure, also one stored in a variable, so it covers every use of `filter`. The fix \
             only renames the command; the closure stays as it is.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/blog/2025-06-10-nushell_0_105_0.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn diagnostic_tags(&self) -> &'static [DiagnosticTag] {
        &[DiagnosticTag::Deprecated]
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        command_renames(context, MIGRATIONS)
    }

    fn fix(&self, _context: &LintContext, rename: &Self::FixInput<'_>) -> Option<Fix> {
        Some(rename.fix())
    }
}

pub static RULE: &dyn Rule = &FilterToWhere;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
