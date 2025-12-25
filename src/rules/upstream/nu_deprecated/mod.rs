use nu_protocol::ParseWarning;

use crate::{
    LintLevel, NU_PARSER_VERSION,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct NuDeprecated;

impl DetectFix for NuDeprecated {
    type FixInput = ();

    fn id(&self) -> &'static str {
        "nu_deprecated"
    }

    fn explanation(&self) -> &'static str {
        "Nushell parser detected deprecated command or flag usage"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
        Self::no_fix(
            context
                .working_set
                .parse_warnings
                .iter()
                .map(|warning| {
                    let ParseWarning::Deprecated {
                        label, span, help, ..
                    } = warning;
                    let mut violation = Detection::from_global_span(label.clone(), *span)
                        .with_primary_label("deprecated");
                    let nu_version_note =
                        format!("This linter was compiled against Nu {NU_PARSER_VERSION}");
                    if let Some(help_text) = help {
                        violation =
                            violation.with_help(help_text.clone() + "\n\n" + &nu_version_note);
                    } else {
                        violation = violation.with_help(nu_version_note);
                    }
                    violation
                })
                .collect(),
        )
    }
}

pub static RULE: &dyn Rule = &NuDeprecated;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
