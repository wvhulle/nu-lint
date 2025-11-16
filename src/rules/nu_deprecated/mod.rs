use nu_protocol::ParseWarning;

use crate::{context::LintContext, rule::Rule, violation::Violation};

fn check(context: &LintContext) -> Vec<Violation> {
    // Iterate over parse warnings and convert deprecation warnings to violations
    context
        .working_set
        .parse_warnings
        .iter()
        .map(|warning| {
            let ParseWarning::Deprecated {
                dep_type,
                label,
                span,
                help,
                ..
            } = warning;
            let mut violation = Violation::new(
                "nu_deprecated",
                format!("{dep_type} deprecated: {label}"),
                *span,
            );
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
mod ignore_good;
