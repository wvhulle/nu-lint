use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

const MAX_POSITIONAL: usize = 2;

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context
        .new_user_functions()
        .filter_map(|(_, decl)| {
            let signature = decl.signature();

            // Count only positional parameters (not flags)
            let positional_count = signature.required_positional.len()
                + signature.optional_positional.len()
                + usize::from(signature.rest_positional.is_some());

            // Only create violation if count exceeds threshold
            (positional_count > MAX_POSITIONAL).then(|| {
                RuleViolation::new_dynamic(
                    "max_positional_params",
                    format!(
                        "Command has {positional_count} positional parameters, should have ≤ \
                         {MAX_POSITIONAL}"
                    ),
                    context.find_declaration_span(&signature.name),
                )
                .with_suggestion_static(
                    "Consider using named flags (--flag) for parameters beyond the first 2",
                )
            })
        })
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "max_positional_params",
        RuleCategory::CodeQuality,
        Severity::Warning,
        "Custom commands should have ≤ 2 positional parameters",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
