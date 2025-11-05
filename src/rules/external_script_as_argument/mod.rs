use nu_protocol::SyntaxShape;

use crate::{
    ast::block::BlockExt,
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

/// Check if a parameter is a string-like type that could be used as a script
/// path
const fn is_string_parameter(param: &nu_protocol::PositionalArg) -> bool {
    matches!(
        param.shape,
        SyntaxShape::String | SyntaxShape::Filepath | SyntaxShape::Any
    )
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    // Find the main function's body block
    let function_bodies = context.collect_function_definitions();
    let main_block_id = function_bodies
        .iter()
        .find_map(|(block_id, name)| (name.as_str() == "main").then_some(*block_id));

    let Some(main_block_id) = main_block_id else {
        return Vec::new();
    };

    // Find all user functions and filter for "main"
    context
        .new_user_functions()
        .find(|(_, decl)| decl.signature().name == "main")
        .and_then(|(_, decl)| {
            let signature = decl.signature();

            // Collect string-type parameters that are used as external commands
            signature
                .required_positional
                .iter()
                .chain(&signature.optional_positional)
                .filter(|param| is_string_parameter(param))
                .filter_map(|param| param.var_id.map(|var_id| (var_id, &param.name)))
                .find(|(var_id, _)| {
                    main_block_id.contains_external_call_with_variable(*var_id, context)
                })
                .map(|(_, param_name)| {
                    RuleViolation::new_dynamic(
                        "external_script_as_argument",
                        format!(
                            "Main function parameter '{param_name}' is used as an external \
                             command. This is an anti-pattern."
                        ),
                        context.find_declaration_span(&signature.name),
                    )
                    .with_suggestion_dynamic(format!(
                        "Instead of passing '{param_name}' as a script path argument, define the \
                         functionality as a function in the same file. This makes the code more \
                         maintainable and testable. For example: 'def {param_name} [] {{ ... }}' \
                         and call it directly in main."
                    ))
                })
        })
        .into_iter()
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "external_script_as_argument",
        RuleCategory::CodeQuality,
        Severity::Warning,
        "Avoid passing external scripts as arguments to main; define them as functions instead",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
