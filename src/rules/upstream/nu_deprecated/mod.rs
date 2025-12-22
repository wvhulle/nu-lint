use nu_protocol::ParseWarning;

use crate::{LintLevel, NU_PARSER_VERSION, context::LintContext, rule::Rule, violation::Violation};

fn check(context: &LintContext) -> Vec<Violation> {
    context
        .working_set
        .parse_warnings
        .iter()
        .map(|warning| {
            let ParseWarning::Deprecated {
                label, span, help, ..
            } = warning;
            let mut violation =
                Violation::new(label.clone(), *span).with_primary_label("deprecated");
            let nu_version_note =
                format!("This linter was compiled against Nu {NU_PARSER_VERSION}");
            if let Some(help_text) = help {
                violation = violation.with_help(help_text.clone() + "\n\n" + &nu_version_note);
            } else {
                violation = violation.with_help(nu_version_note);
            }
            violation
        })
        .collect()
}

pub const fn rule() -> Rule {
    Rule::new(
        "nu_deprecated",
        "Nushell parser detected deprecated command or flag usage",
        check,
        LintLevel::Warning,
    )
    .with_doc_url("https://www.nushell.sh/book/")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
