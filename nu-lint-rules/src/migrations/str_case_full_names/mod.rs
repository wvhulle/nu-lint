use nu_lint_analysis::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, DiagnosticTag, Fix},
};

use super::deprecated_usage::{CommandRename, MigratedDeprecation, command_renames};

pub const MIGRATIONS: &[MigratedDeprecation] = &[
    MigratedDeprecation::renamed_command("str upcase", "str uppercase"),
    MigratedDeprecation::renamed_command("str downcase", "str lowercase"),
];

struct StrCaseFullNames;

impl DetectFix for StrCaseFullNames {
    type FixInput<'a> = CommandRename;

    fn id(&self) -> &'static str {
        "str_case_full_names"
    }

    fn short_description(&self) -> &'static str {
        "Deprecated `str upcase` or `str downcase` renamed"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "`str upcase` and `str downcase` were deprecated in Nushell 0.114.0 in favour of `str \
             uppercase` and `str lowercase`. The behaviour is identical, so the fix only renames \
             the command and keeps its arguments.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/blog/2026-07-04-nushell_v0_114_0.html")
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

pub static RULE: &dyn Rule = &StrCaseFullNames;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
