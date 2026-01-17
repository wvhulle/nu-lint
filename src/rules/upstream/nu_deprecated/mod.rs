use nu_protocol::ParseWarning;

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix},
};

mod enhancers;

struct NuDeprecated;

impl DetectFix for NuDeprecated {
    type FixInput<'a> = Option<Fix>;

    fn id(&self) -> &'static str {
        "nu_deprecated"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "nu-lint expects a specific version. If your installed version differs, this may \
             cause false positives. Check that your version matches the expected version to avoid \
             incorrect warnings.",
        )
    }

    fn short_description(&self) -> &'static str {
        "Parser detected deprecated command or flag usage"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .working_set
            .parse_warnings
            .iter()
            .map(|warning| {
                let ParseWarning::Deprecated {
                    label, span, help, ..
                } = warning;

                let enhancement = enhancers::enhance(warning, context);

                // Build message with notes appended
                let mut message = help
                    .as_ref()
                    .map_or_else(|| label.clone(), |h| format!("{label}. {h}"));
                if let Some(ref enh) = enhancement {
                    for note in &enh.notes {
                        message.push_str("\n\nNote: ");
                        message.push_str(note);
                    }
                }

                let mut detection =
                    Detection::from_global_span(message, *span).with_primary_label("deprecated");

                // Apply extra labels from enhancement
                if let Some(ref enh) = enhancement {
                    for (label_span, label_text) in &enh.extra_labels {
                        detection = detection.with_extra_label(label_text.clone(), *label_span);
                    }
                }

                let fix = enhancement.and_then(|e| e.fix);
                (detection, fix)
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        fix_data.clone()
    }
}

pub static RULE: &dyn Rule = &NuDeprecated;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
