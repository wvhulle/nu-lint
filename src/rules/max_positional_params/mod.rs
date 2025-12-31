use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};
const MAX_POSITIONAL: usize = 2;

struct MaxPositionalParams;

impl DetectFix for MaxPositionalParams {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "max_positional_params"
    }

    fn explanation(&self) -> &'static str {
        "Custom commands should have ≤ 2 positional parameters"
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let detections = context
            .new_user_functions()
            .filter_map(|(_, decl)| {
                let signature = decl.signature();
                log::debug!("Checking command '{}'", signature.name);
                // Count only positional parameters (not flags)
                let positional_count = signature.required_positional.len()
                    + signature.optional_positional.len()
                    + usize::from(signature.rest_positional.is_some());
                log::debug!(
                    "Command '{}' has {positional_count} positional parameters",
                    signature.name
                );
                // Only create violation if count exceeds threshold
                (positional_count > MAX_POSITIONAL).then(|| {
                    log::warn!(
                        "Command '{}' exceeds max positional parameters with {positional_count} \
                         parameters",
                        signature.name
                    );
                    let name_span = context.find_declaration_span(&signature.name);
                    Detection::from_file_span(
                        format!(
                            "Command has {positional_count} positional parameters, should have ≤ \
                             {MAX_POSITIONAL}"
                        ),
                        name_span,
                    )
                    .with_primary_label(format!("{positional_count} positional params"))
                    .with_help(
                        "Consider using named flags (--flag) for parameters beyond the first 2",
                    )
                })
            })
            .collect();
        Self::no_fix(detections)
    }
}

pub static RULE: &dyn Rule = &MaxPositionalParams;
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
