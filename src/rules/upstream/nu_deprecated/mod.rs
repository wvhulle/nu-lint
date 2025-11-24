use nu_protocol::ParseWarning;

use crate::{context::LintContext, rule::Rule, violation::Violation};

fn check(context: &LintContext) -> Vec<Violation> {
    context
        .working_set
        .parse_warnings
        .iter()
        .map(|warning| {
            let ParseWarning::Deprecated {
                label, span, help, ..
            } = warning;
            let mut violation = Violation::new("nu_deprecated", label.clone(), *span);
            if let Some(help_text) = help {
                violation = violation.with_help(help_text.clone());
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
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
