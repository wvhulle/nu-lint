use nu_lint_analysis::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, DiagnosticTag, Fix, Replacement},
};
use nu_protocol::Span;

use super::deprecated_usage::{MigratedCall, MigratedDeprecation, migrated_calls};

pub const MIGRATIONS: &[MigratedDeprecation] = &[
    MigratedDeprecation::renamed_flag("get", "ignore-errors", "optional"),
    MigratedDeprecation::renamed_flag("select", "ignore-errors", "optional"),
    MigratedDeprecation::renamed_flag("reject", "ignore-errors", "optional"),
];

struct OptionalFlagRename {
    flag_span: Span,
    successor_flag: String,
}

fn optional_flag_rename(call: &MigratedCall, context: &LintContext) -> Option<OptionalFlagRename> {
    let deprecated_flag = call.flag.as_ref()?;
    let signature = context.working_set.get_decl(call.decl_id).signature();
    let successor = signature.get_long_flag(call.migration.successor?)?;
    let successor_flag = match successor.short {
        Some(short) if !deprecated_flag.is_written_long => format!("-{short}"),
        _ => format!("--{}", successor.long),
    };
    Some(OptionalFlagRename {
        flag_span: deprecated_flag.span,
        successor_flag,
    })
}

struct IgnoreErrorsToOptional;

impl DetectFix for IgnoreErrorsToOptional {
    type FixInput<'a> = OptionalFlagRename;

    fn id(&self) -> &'static str {
        "ignore_errors_to_optional"
    }

    fn short_description(&self) -> &'static str {
        "Deprecated `--ignore-errors` flag renamed to `--optional`"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "The `--ignore-errors` (`-i`) flag of `get`, `select` and `reject` was renamed to \
             `--optional` (`-o`) in Nushell 0.106.0, because it makes missing cell paths optional \
             instead of hiding errors. The fix keeps the short or long spelling that was used.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/blog/2025-07-23-nushell_0_106_0.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn diagnostic_tags(&self) -> &'static [DiagnosticTag] {
        &[DiagnosticTag::Deprecated]
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        migrated_calls(context, MIGRATIONS)
            .iter()
            .filter_map(|call| optional_flag_rename(call, context))
            .map(|rename| {
                let detection = Detection::from_global_span(
                    "`--ignore-errors` is deprecated in favour of `--optional`",
                    rename.flag_span,
                )
                .with_primary_label("deprecated");
                (detection, rename)
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, rename: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix {
            explanation: format!("Rename the flag to `{}`", rename.successor_flag).into(),
            replacements: vec![Replacement::new(
                rename.flag_span,
                rename.successor_flag.clone(),
            )],
        })
    }
}

pub static RULE: &dyn Rule = &IgnoreErrorsToOptional;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
