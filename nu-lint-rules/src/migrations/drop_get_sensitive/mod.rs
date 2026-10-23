use nu_lint_analysis::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, DiagnosticTag, Fix, Replacement},
};
use nu_protocol::Span;

use super::deprecated_usage::{MigratedDeprecation, migrated_calls};

pub const MIGRATIONS: &[MigratedDeprecation] =
    &[MigratedDeprecation::removed_flag("get", "sensitive")];

struct SensitiveFlagRemoval {
    span_with_leading_space: Span,
}

struct DropGetSensitive;

impl DetectFix for DropGetSensitive {
    type FixInput<'a> = SensitiveFlagRemoval;

    fn id(&self) -> &'static str {
        "drop_get_sensitive"
    }

    fn short_description(&self) -> &'static str {
        "Deprecated `get --sensitive` flag has no effect"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Cell paths have been case-sensitive by default since Nushell 0.105.0, so `get \
             --sensitive` no longer changes anything and is deprecated. To access one member \
             case-insensitively, add `!` after it in the cell path instead.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/blog/2025-06-10-nushell_0_105_0.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn diagnostic_tags(&self) -> &'static [DiagnosticTag] {
        &[DiagnosticTag::Deprecated, DiagnosticTag::Unnecessary]
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        migrated_calls(context, MIGRATIONS)
            .into_iter()
            .filter_map(|call| call.flag)
            .map(|flag| {
                let detection = Detection::from_global_span(
                    "`--sensitive` is deprecated because cell paths are case-sensitive by default",
                    flag.span,
                )
                .with_primary_label("no effect");
                let removal = SensitiveFlagRemoval {
                    span_with_leading_space: flag.span_with_leading_space,
                };
                (detection, removal)
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, removal: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix {
            explanation: "Remove the flag".into(),
            replacements: vec![Replacement::new(removal.span_with_leading_space, "")],
        })
    }
}

pub static RULE: &dyn Rule = &DropGetSensitive;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
