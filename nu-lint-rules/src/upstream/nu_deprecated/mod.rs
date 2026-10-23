use nu_lint_analysis::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, DiagnosticTag},
};
use nu_protocol::ParseWarning;

use crate::migrations::deprecated_usage::migrated_warning_spans;

struct NuDeprecated;

impl DetectFix for NuDeprecated {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "nu_deprecated"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Forwards every deprecation warning of nu-parser that no rule in the \
             `migrate-deprecated-syntax` group handles. Deprecations with a migration rule are \
             reported only by that rule, even when it is turned off, so each warning appears \
             exactly once. The warnings depend on the nu-parser version nu-lint was built against.",
        )
    }

    fn short_description(&self) -> &'static str {
        "Upstream deprecation without a dedicated migration rule"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn diagnostic_tags(&self) -> &'static [DiagnosticTag] {
        &[DiagnosticTag::Deprecated]
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let migrated_spans = migrated_warning_spans(context);
        let detections = context
            .working_set
            .parse_warnings
            .iter()
            .filter_map(|warning| {
                let ParseWarning::Deprecated {
                    label, span, help, ..
                } = warning;
                if migrated_spans.contains(span) {
                    return None;
                }
                let message = help
                    .as_ref()
                    .map_or_else(|| label.clone(), |help| format!("{label} {help}"));
                Some(Detection::from_global_span(message, *span).with_primary_label("deprecated"))
            })
            .collect();
        Self::no_fix(detections)
    }
}

pub static RULE: &dyn Rule = &NuDeprecated;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
