use nu_protocol::SyntaxShape;

use crate::{
    LintLevel, ast::block::BlockExt, context::LintContext, rule::Rule, violation::Violation,
};

/// Check if a parameter is a string-like type that could be used as a script
/// path
const fn is_string_parameter(param: &nu_protocol::PositionalArg) -> bool {
    matches!(
        param.shape,
        SyntaxShape::String | SyntaxShape::Filepath | SyntaxShape::Any
    )
}

/// Generate suggestion message based on function context
fn create_suggestion_message(param_name: &str, function_name: &str) -> String {
    if function_name == "main" {
        format!(
            "Instead of passing '{param_name}' as a script path argument, define the \
             functionality as a function in the same file. This makes the code more maintainable \
             and testable. For example: 'def {param_name}-handler [] {{ ... }}' and call it \
             directly in main."
        )
    } else {
        format!(
            "Instead of passing '{param_name}' as a script path argument to function \
             '{function_name}', consider defining the external script logic as an internal \
             function. This improves code maintainability and testability."
        )
    }
}

/// Create a violation for a parameter that's used as an external command
fn create_violation(param_name: &str, function_name: &str, context: &LintContext) -> Violation {
    Violation::with_file_span(
        format!(
            "Function '{function_name}' parameter '{param_name}' is used as an external command."
        ),
        context.find_declaration_span(function_name),
    )
    .with_primary_label("using script parameter")
    .with_help(create_suggestion_message(param_name, function_name))
}

fn check(context: &LintContext) -> Vec<Violation> {
    let function_bodies = context.collect_function_definitions();

    context
        .new_user_functions()
        .filter_map(|(_, decl)| {
            let signature = decl.signature();

            // Find the function body block
            function_bodies
                .iter()
                .find_map(|(block_id, name)| (name == &signature.name).then_some(*block_id))
                .map(|function_block_id| (signature, function_block_id))
        })
        .flat_map(|(signature, function_block_id)| {
            let function_block = context.working_set.get_block(function_block_id);
            signature
                .required_positional
                .iter()
                .chain(&signature.optional_positional)
                .filter(|param| is_string_parameter(param))
                .filter_map(|param| param.var_id.map(|var_id| (param, var_id)))
                .filter(|(_, var_id)| {
                    function_block.contains_external_call_with_variable(*var_id, context)
                })
                .map(|(param, _)| create_violation(&param.name, &signature.name, context))
                .collect::<Vec<_>>()
        })
        .collect()
}

pub const fn rule() -> Rule {
    Rule::new(
        "external_script_as_argument",
        "Avoid passing external scripts as arguments to custom commands; define them as functions \
         instead",
        check,
        LintLevel::Warning,
    )
    .with_doc_url("https://www.nushell.sh/book/modules.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
