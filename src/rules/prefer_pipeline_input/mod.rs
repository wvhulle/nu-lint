use nu_protocol::{
    Category, SyntaxShape, VarId,
    ast::{Expr, Pipeline},
};

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

/// Check if a parameter is a data type that would benefit from pipeline input
fn is_data_type_parameter(param: &nu_protocol::PositionalArg) -> bool {
    log::debug!("Parameter '{}' has shape: {:?}", param.name, param.shape);
    matches!(
        param.shape,
        SyntaxShape::List(_)
            | SyntaxShape::Table(_)
            | SyntaxShape::Record(_)
            | SyntaxShape::String // String data processing
            | SyntaxShape::Any // Untyped parameters might be data
    )
}

/// Check if expression references a specific variable
fn references_variable(expr: &nu_protocol::ast::Expression, var_id: VarId) -> bool {
    match &expr.expr {
        Expr::Var(id) => *id == var_id,
        Expr::FullCellPath(cell_path) if cell_path.tail.is_empty() => {
            matches!(&cell_path.head.expr, Expr::Var(id) if *id == var_id)
        }
        _ => false,
    }
}

/// Check if a pipeline uses parameter as first element and has data transformation commands
fn pipeline_uses_parameter_for_data_operations(
    pipeline: &Pipeline,
    param_var_id: VarId,
    context: &LintContext,
) -> bool {
    if pipeline.elements.len() < 2 {
        return false;
    }

    // Check if first element references our parameter
    let first_element = &pipeline.elements[0];
    if !references_variable(&first_element.expr, param_var_id) {
        return false;
    }

    // Check if subsequent elements are data transformation commands
    for element in &pipeline.elements[1..] {
        if let Expr::Call(call) = &element.expr.expr {
            let decl = context.working_set.get_decl(call.decl_id);
            let category = decl.signature().category;

            // Check if command is in data processing categories
            if matches!(
                category,
                Category::Filters | Category::Math | Category::Formats | Category::Strings
            ) {
                return true;
            }
        }
    }

    false
}

/// Check if any block in the function uses parameter for data operations using AST
fn likely_uses_parameter_for_data_operations(param_var_id: VarId, context: &LintContext) -> bool {
    // Use AST traversal to find parameter usage in pipelines
    let violations = context.collect_rule_violations(|expr, ctx| {
        match &expr.expr {
            Expr::Block(block_id) | Expr::Closure(block_id) => {
                let block = ctx.working_set.get_block(*block_id);
                for pipeline in &block.pipelines {
                    if pipeline_uses_parameter_for_data_operations(pipeline, param_var_id, ctx) {
                        return vec![RuleViolation::new_static(
                            "temp_marker",
                            "Found data operation usage",
                            expr.span,
                        )];
                    }
                }
            }
            _ => {}
        }
        vec![]
    });

    !violations.is_empty()
}

/// Extract the function body from its declaration span for generating specific suggestions
fn extract_function_body(decl_name: &str, _param_name: &str, context: &LintContext) -> Option<String> {
    // Find the declaration span and extract the body
    let decl_span = context.find_declaration_span(decl_name);
    let contents = String::from_utf8_lossy(context.working_set.get_span_contents(decl_span));

    // Look for the function body between braces
    if let Some(start) = contents.find('{') {
        if let Some(end) = contents.rfind('}') {
            let body = &contents[start+1..end].trim();
            return Some(body.to_string());
        }
    }

    None
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let user_functions: Vec<_> = context.new_user_functions().collect();
    log::debug!(
        "prefer_pipeline_input: Found {} user functions",
        user_functions.len()
    );

    user_functions
        .into_iter()
        .filter_map(|(_, decl)| {
            let signature = decl.signature();
            log::debug!("Checking function: {} with {} required, {} optional, rest: {:?}",
                       signature.name,
                       signature.required_positional.len(),
                       signature.optional_positional.len(),
                       signature.rest_positional.is_some());

            // Only consider functions with exactly one required positional parameter
            if signature.required_positional.len() != 1
                || !signature.optional_positional.is_empty()
                || signature.rest_positional.is_some() {
                log::debug!("Skipping {} - wrong parameter count", signature.name);
                return None;
            }

            let param = &signature.required_positional[0];

            // Check if the parameter is a data type that would benefit from pipeline input
            if !is_data_type_parameter(param) {
                return None;
            }

            // Get parameter variable ID to track usage
            let param_var_id = param.var_id?;

            // Check if the function body uses the parameter for data operations
            if !likely_uses_parameter_for_data_operations(param_var_id, context) {
                log::debug!("Function {} with param {} does not use parameter for data operations", signature.name, param.name);
                return None;
            }

            // Generate a specific suggestion based on the function body
            let suggestion = if let Some(function_body) = extract_function_body(&signature.name, &param.name, context) {
                let param_var = format!("${}", param.name);

                // Check if the parameter is used at the start of a pipeline (can omit $in)
                let suggested_body = if function_body.trim_start().starts_with(&param_var)
                    && function_body.contains(" | ") {
                    // Parameter is at start of pipeline - can omit $in
                    function_body.replacen(&format!("{} | ", param_var), "", 1)
                } else {
                    // Parameter is used elsewhere - need explicit $in
                    function_body.replace(&param_var, "$in")
                };

                format!(
                    "Change to: def {} [] {{ {} }}. Remove the '{}' parameter and use pipeline input.",
                    signature.name, suggested_body.trim(), param.name
                )
            } else {
                format!(
                    "Remove the '{}' parameter and use pipeline input. Change to: def {} [] {{ ... }}",
                    param.name, signature.name
                )
            };

            Some(RuleViolation::new_dynamic(
                "prefer_pipeline_input",
                format!(
                    "Custom command '{}' with single data parameter '{}' should use pipeline input ($in) instead",
                    signature.name, param.name
                ),
                context.find_declaration_span(&signature.name),
            )
            .with_suggestion_dynamic(suggestion))
        })
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_pipeline_input",
        RuleCategory::Idioms,
        Severity::Info,
        "Custom commands with single data parameters should use pipeline input for better composability",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
