use const_format::formatcp;
use nu_protocol::ParseWarning;

use crate::{
    LintLevel, NU_PARSER_VERSION,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};
struct NuDeprecated;

impl DetectFix for NuDeprecated {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "nu_deprecated"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(formatcp!(
            "nu-lint expects Nushell {NU_PARSER_VERSION}. If your installed version differs, this \
             may cause false positives. Check that your Nushell version matches the expected \
             version to avoid incorrect warnings."
        ))
    }

    fn short_description(&self) -> &'static str {
        "Nushell parser detected deprecated command or flag usage"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(
            context
                .working_set
                .parse_warnings
                .iter()
                .map(|warning| {
                    let ParseWarning::Deprecated { label, span, .. } = warning;
                    Detection::from_global_span(label.clone(), *span)
                        .with_primary_label("deprecated")
                })
                .collect(),
        )
    }
}

pub static RULE: &dyn Rule = &NuDeprecated;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
