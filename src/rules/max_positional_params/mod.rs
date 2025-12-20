use crate::{context::LintContext, rule::Rule, violation::Violation};
const MAX_POSITIONAL: usize = 2;
fn check(context: &LintContext) -> Vec<Violation> {
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
                let name_span = context.find_declaration_span(&signature.name);
                Violation::with_file_span(
                    format!(
                        "Command has {positional_count} positional parameters, should have ≤ \
                         {MAX_POSITIONAL}"
                    ),
                    name_span,
                )
                .with_primary_label(format!("{positional_count} positional params"))
                .with_help("Consider using named flags (--flag) for parameters beyond the first 2")
            })
        })
        .collect()
}
pub const fn rule() -> Rule {
    Rule::new(
        "max_positional_params",
        "Custom commands should have ≤ 2 positional parameters",
        check,
    )
    .with_doc_url(
        "https://www.nushell.sh/book/style_guide.html#options-and-parameters-of-custom-commands",
    )
}
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
