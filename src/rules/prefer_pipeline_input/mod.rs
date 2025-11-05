use nu_protocol::{Category, SyntaxShape, VarId, ast::Expr};

use crate::{
    ast::{CallExt, ExpressionExt, SpanExt},
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{self, RuleViolation, Severity},
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

/// Check if expression references a specific variable (comprehensive)
fn references_variable(expr: &nu_protocol::ast::Expression, var_id: VarId) -> bool {
    match &expr.expr {
        Expr::Var(id) => *id == var_id,
        Expr::FullCellPath(cell_path) => {
            matches!(&cell_path.head.expr, Expr::Var(id) if *id == var_id)
        }
        _ => false,
    }
}

fn is_data_processing_command(decl_name: &str, category: &Category) -> bool {
    // Check by category first
    if matches!(
        category,
        Category::Filters
            | Category::Math
            | Category::Formats
            | Category::Strings
            | Category::Conversions
            | Category::Database
    ) {
        return true;
    }

    // Check specific commands that do data processing regardless of category
    matches!(
        decl_name,
        "each"
            | "where"
            | "select"
            | "sort"
            | "sort-by"
            | "group-by"
            | "reduce"
            | "length"
            | "first"
            | "last"
            | "skip"
            | "take"
            | "unique"
            | "uniq"
            | "flatten"
            | "transpose"
            | "reverse"
            | "shuffle"
            | "update"
            | "upsert"
            | "insert"
            | "append"
            | "prepend"
            | "get"
            | "drop"
            | "enumerate"
            | "chunks"
            | "split-by"
            | "merge"
            | "zip"
            | "find"
            | "any"
            | "all"
            | "empty?"
            | "is-empty"
            | "is-not-empty"
            | "describe"
            | "compact"
            | "collect"
            | "par-each"
            | "rotate"
            | "roll"
    )
}

/// Analyze parameter usage patterns to determine if it's used for data
/// operations
#[allow(clippy::struct_excessive_bools)]
#[derive(Default)]
struct ParameterUsageAnalysis {
    /// Parameter used as pipeline input (first element in pipeline)
    used_as_pipeline_input: bool,
    /// Parameter used with data processing commands
    used_with_data_commands: bool,
    /// Parameter used for data access (field access, etc.)
    used_for_data_access: bool,
    /// Parameter used in generation/configuration contexts
    used_for_generation: bool,
    /// Parameter used with file/path operations
    used_with_file_operations: bool,
    /// Total usage count
    usage_count: usize,
}

impl ParameterUsageAnalysis {
    /// Determine if this usage pattern suggests data operations
    fn suggests_data_operations(&self) -> bool {
        // Must have some data-related usage
        let has_data_usage = self.used_as_pipeline_input
            || self.used_with_data_commands
            || self.used_for_data_access;

        // Shouldn't be primarily used for file operations or generation
        let primarily_non_data = self.used_with_file_operations && !has_data_usage;

        has_data_usage && !primarily_non_data
    }
}

/// Comprehensive analysis of how a parameter is used in a function
fn analyze_parameter_usage(param_var_id: VarId, context: &LintContext) -> ParameterUsageAnalysis {
    let mut analysis = ParameterUsageAnalysis::default();

    log::debug!("Starting analysis for parameter var_id: {param_var_id:?}");
    log::debug!("Main AST has {} pipelines", context.ast.pipelines.len());

    // Analyze the main AST block
    for (i, pipeline) in context.ast.pipelines.iter().enumerate() {
        log::debug!(
            "Analyzing main pipeline {}: {} elements",
            i,
            pipeline.elements.len()
        );
        analyze_pipeline_for_parameter_usage(pipeline, param_var_id, context, &mut analysis);
    }

    // Use AST traversal to find more complex usage patterns
    let nested_analysis_count = context
        .collect_rule_violations(|expr, ctx| {
            log::debug!("Traversing expression: {:?}", expr.expr);
            analyze_expression_for_parameter_usage_impl(expr, param_var_id, ctx, &analysis);
            Vec::new() // We're not collecting violations here, just analyzing
        })
        .len();

    log::debug!("Traversed {nested_analysis_count} nested expressions");
    log::debug!(
        "Analysis result: used_as_pipeline_input={}, used_with_data_commands={}, \
         used_for_data_access={}, usage_count={}",
        analysis.used_as_pipeline_input,
        analysis.used_with_data_commands,
        analysis.used_for_data_access,
        analysis.usage_count
    );

    analysis
}

/// Implementation that doesn't need mutable access to analysis (for closure
/// compatibility)
fn analyze_expression_for_parameter_usage_impl(
    expr: &nu_protocol::ast::Expression,
    _param_var_id: VarId,
    ctx: &LintContext,
    _analysis: &ParameterUsageAnalysis,
) {
    // This function will be used for complex nested analysis if needed
    // For now, we handle most cases in the main pipeline analysis above
    match &expr.expr {
        Expr::Block(block_id) | Expr::Closure(block_id) | Expr::Subexpression(block_id) => {
            let block = ctx.working_set.get_block(*block_id);
            log::debug!(
                "Found nested block with {} pipelines",
                block.pipelines.len()
            );
            for (i, pipeline) in block.pipelines.iter().enumerate() {
                log::debug!(
                    "  Nested pipeline {}: {} elements",
                    i,
                    pipeline.elements.len()
                );
                // Additional nested analysis could go here if needed
                // For now, the main analysis covers most cases
            }
        }
        Expr::Call(call) => {
            let decl = ctx.working_set.get_decl(call.decl_id);
            log::debug!("Found call to: {}", decl.signature().name);
        }
        _ => {
            // Track other usage patterns if needed
        }
    }
}

/// Helper function to categorize command usage
fn categorize_command_usage(
    decl_name: &str,
    category: &Category,
    analysis: &mut ParameterUsageAnalysis,
) {
    if is_data_processing_command(decl_name, category) {
        analysis.used_with_data_commands = true;
    } else if is_file_operation_command(decl_name) {
        analysis.used_with_file_operations = true;
    }
}

/// Analyze a pipeline for parameter usage
fn analyze_pipeline_for_parameter_usage(
    pipeline: &nu_protocol::ast::Pipeline,
    param_var_id: VarId,
    ctx: &LintContext,
    analysis: &mut ParameterUsageAnalysis,
) {
    if pipeline.elements.is_empty() {
        return;
    }

    // Check if parameter is used as first element (pipeline input)
    let first_element = &pipeline.elements[0];
    if references_variable(&first_element.expr, param_var_id) {
        analysis.used_as_pipeline_input = true;
        analysis.usage_count += 1;

        // Check if subsequent elements are data processing commands
        for element in &pipeline.elements[1..] {
            if let Expr::Call(call) = &element.expr.expr {
                let decl = ctx.working_set.get_decl(call.decl_id);
                let decl_name = &decl.signature().name;
                let category = decl.signature().category;
                categorize_command_usage(decl_name, &category, analysis);
            }
        }
    }

    // Also check each element for parameter usage in arguments
    for element in &pipeline.elements {
        if let Expr::Call(call) = &element.expr.expr {
            analyze_call_for_parameter_usage(call, param_var_id, ctx, analysis);
        }
    }
}

/// Check if a command is a file operation
fn is_file_operation_command(decl_name: &str) -> bool {
    matches!(
        decl_name,
        "open" | "save" | "load" | "cp" | "mv" | "rm" | "mkdir" | "touch" | "ls"
    )
}

/// Analyze a function call for parameter usage
fn analyze_call_for_parameter_usage(
    call: &nu_protocol::ast::Call,
    param_var_id: VarId,
    ctx: &LintContext,
    analysis: &mut ParameterUsageAnalysis,
) {
    let decl = ctx.working_set.get_decl(call.decl_id);
    let decl_name = &decl.signature().name;
    let category = decl.signature().category;

    // Check if any arguments reference our parameter
    let param_used_in_call = call
        .arguments
        .iter()
        .any(|arg| argument_references_variable(arg, param_var_id));

    if param_used_in_call {
        analysis.usage_count += 1;

        // Categorize the command usage
        if is_data_processing_command(decl_name, &category) {
            analysis.used_with_data_commands = true;
        } else if is_file_operation_command(decl_name) {
            analysis.used_with_file_operations = true;
        } else if is_generation_command(decl_name) {
            analysis.used_for_generation = true;
        }
    }
}

/// Check if a command is primarily for generation/creation
fn is_generation_command(decl_name: &str) -> bool {
    matches!(
        decl_name,
        "range"
            | "seq"
            | "random"
            | "date"
            | "now"
            | "create"
            | "make"
            | "generate"
            | "build"
            | "repeat"
    )
}

/// Check if an argument references a variable
fn argument_references_variable(arg: &nu_protocol::ast::Argument, var_id: VarId) -> bool {
    match arg {
        nu_protocol::ast::Argument::Positional(expr)
        | nu_protocol::ast::Argument::Named((_, _, Some(expr)))
        | nu_protocol::ast::Argument::Unknown(expr)
        | nu_protocol::ast::Argument::Spread(expr) => expression_contains_variable(expr, var_id),
        nu_protocol::ast::Argument::Named(_) => false,
    }
}

/// Recursively check if an expression contains a variable reference
fn expression_contains_variable(expr: &nu_protocol::ast::Expression, var_id: VarId) -> bool {
    match &expr.expr {
        Expr::Var(id) => *id == var_id,
        Expr::FullCellPath(cell_path) => expression_contains_variable(&cell_path.head, var_id),
        Expr::BinaryOp(left, _op, right) => {
            expression_contains_variable(left, var_id)
                || expression_contains_variable(right, var_id)
        }
        Expr::UnaryNot(inner) => expression_contains_variable(inner, var_id),
        Expr::Call(call) => call
            .arguments
            .iter()
            .any(|arg| argument_references_variable(arg, var_id)),
        Expr::List(items) => items.iter().any(|item| {
            let expr = match item {
                nu_protocol::ast::ListItem::Item(e) | nu_protocol::ast::ListItem::Spread(_, e) => e,
            };
            expression_contains_variable(expr, var_id)
        }),
        Expr::Table(table) => {
            // Check columns
            table.columns.iter().any(|col| expression_contains_variable(col, var_id))
                // Check rows
                || table.rows.iter().any(|row| {
                    row.iter().any(|cell| expression_contains_variable(cell, var_id))
                })
        }
        Expr::Record(items) => items.iter().any(|item| match item {
            nu_protocol::ast::RecordItem::Pair(key, val) => {
                expression_contains_variable(key, var_id)
                    || expression_contains_variable(val, var_id)
            }
            nu_protocol::ast::RecordItem::Spread(_, expr) => {
                expression_contains_variable(expr, var_id)
            }
        }),
        _ => false,
    }
}

/// Extract the function body from its declaration span for generating specific
/// suggestions
fn extract_function_body(
    decl_name: &str,
    _param_name: &str,
    context: &LintContext,
) -> Option<String> {
    // Find the function definition in the AST
    context
        .ast
        .pipelines
        .iter()
        .flat_map(|pipeline| &pipeline.elements)
        .filter_map(|element| element.expr.extract_call())
        .find_map(|call| {
            let (block_id, name) = call.extract_function_definition(context)?;
            if name == decl_name {
                let block = context.working_set.get_block(block_id);
                let block_span = block.span?;
                let body_text = block_span.text(context);

                // Remove outer braces from the body
                let trimmed = body_text.trim();
                if trimmed.starts_with('{') && trimmed.ends_with('}') {
                    let inner = &trimmed[1..trimmed.len() - 1];
                    Some(inner.trim().to_string())
                } else {
                    Some(trimmed.to_string())
                }
            } else {
                None
            }
        })
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    log::debug!("prefer_pipeline_input: Starting rule check");

    // First try the engine-based approach for registered functions
    let user_functions: Vec<_> = context.new_user_functions().collect();
    log::debug!("Found {} registered user functions", user_functions.len());

    // Also check function definitions from AST (for test cases and unregistered
    // functions)
    let function_definitions = context.collect_function_definitions();
    log::debug!(
        "Found {} function definitions in AST",
        function_definitions.len()
    );

    let mut violations = Vec::new();

    // Process registered functions
    for (_, decl) in user_functions {
        if let Some(violation) = analyze_function_from_signature(&decl.signature(), context) {
            violations.push(violation);
        }
    }

    // Process AST function definitions
    for (block_id, function_name) in function_definitions {
        if let Some(violation) = analyze_function_from_ast(block_id, &function_name, context) {
            violations.push(violation);
        }
    }

    violations
}

/// Analyze a function from its signature (for registered functions)
fn analyze_function_from_signature(
    signature: &nu_protocol::Signature,
    context: &LintContext,
) -> Option<RuleViolation> {
    log::debug!(
        "Checking registered function: {} with {} required, {} optional, rest: {:?}",
        signature.name,
        signature.required_positional.len(),
        signature.optional_positional.len(),
        signature.rest_positional.is_some()
    );

    // Only consider functions with exactly one required positional parameter
    if signature.required_positional.len() != 1
        || !signature.optional_positional.is_empty()
        || signature.rest_positional.is_some()
    {
        log::debug!("Skipping {} - wrong parameter count", signature.name);
        return None;
    }

    let param = &signature.required_positional[0];

    // Check if the parameter is a data type that would benefit from pipeline input
    if !is_data_type_parameter(param) {
        log::debug!(
            "Skipping {} - parameter '{}' is not a data type",
            signature.name,
            param.name
        );
        return None;
    }

    // Get parameter variable ID to track usage
    let param_var_id = param.var_id?;
    log::debug!(
        "Function {} parameter '{}' has var_id: {:?}",
        signature.name,
        param.name,
        param_var_id
    );

    // Check if the function body uses the parameter for data operations
    let analysis = analyze_parameter_usage(param_var_id, context);
    if !analysis.suggests_data_operations() {
        log::debug!(
            "Function {} with param {} does not use parameter for data operations",
            signature.name,
            param.name
        );
        return None;
    }

    Some(create_violation(signature, param, context))
}

/// Analyze a function from AST definition (for unregistered functions)
fn analyze_function_from_ast(
    block_id: nu_protocol::BlockId,
    function_name: &str,
    context: &LintContext,
) -> Option<RuleViolation> {
    let block = context.working_set.get_block(block_id);
    let signature = &block.signature;

    log::debug!(
        "Checking AST function: {} with {} required, {} optional, rest: {:?}",
        function_name,
        signature.required_positional.len(),
        signature.optional_positional.len(),
        signature.rest_positional.is_some()
    );

    // Only consider functions with exactly one required positional parameter
    if signature.required_positional.len() != 1
        || !signature.optional_positional.is_empty()
        || signature.rest_positional.is_some()
    {
        log::debug!("Skipping {function_name} - wrong parameter count");
        return None;
    }

    let param = &signature.required_positional[0];

    // Check if the parameter is a data type that would benefit from pipeline input
    if !is_data_type_parameter(param) {
        log::debug!(
            "Skipping {} - parameter '{}' is not a data type",
            function_name,
            param.name
        );
        return None;
    }

    // Get parameter variable ID to track usage
    let param_var_id = param.var_id?;
    log::debug!(
        "Function {} parameter '{}' has var_id: {:?}",
        function_name,
        param.name,
        param_var_id
    );

    // Check if the function body uses the parameter for data operations by
    // analyzing the block directly
    let analysis = analyze_parameter_usage_in_block(param_var_id, block, context);
    if !analysis.suggests_data_operations() {
        log::debug!(
            "Function {} with param {} does not use parameter for data operations",
            function_name,
            param.name
        );
        return None;
    }

    // Create a signature for violation creation
    let mut function_signature = signature.clone();
    function_signature.name = function_name.to_string();
    Some(create_violation(&function_signature, param, context))
}

/// Create a violation for a function that should use pipeline input
fn create_violation(
    signature: &nu_protocol::Signature,
    param: &nu_protocol::PositionalArg,
    context: &LintContext,
) -> violation::RuleViolation {
    // Generate a specific suggestion based on the function body
    let suggestion =
        if let Some(function_body) = extract_function_body(&signature.name, &param.name, context) {
            let param_var = format!("${}", param.name);

            // Check if the parameter is used at the start of a pipeline (can omit $in)
            let suggested_body = if function_body.trim_start().starts_with(&param_var)
                && function_body.contains(" | ")
            {
                // Parameter is at start of pipeline - can omit $in
                function_body.replacen(&format!("{param_var} | "), "", 1)
            } else {
                // Parameter is used elsewhere - need explicit $in
                function_body.replace(&param_var, "$in")
            };

            format!(
                "Change to: def {} [] {{ {} }}. Remove the '{}' parameter and use pipeline input.",
                signature.name,
                suggested_body.trim(),
                param.name
            )
        } else {
            format!(
                "Remove the '{}' parameter and use pipeline input. Change to: def {} [] {{ ... }}",
                param.name, signature.name
            )
        };

    RuleViolation::new_dynamic(
        "prefer_pipeline_input",
        format!(
            "Custom command '{}' with single data parameter '{}' should use pipeline input ($in) \
             instead",
            signature.name, param.name
        ),
        context.find_declaration_span(&signature.name),
    )
    .with_suggestion_dynamic(suggestion)
}

/// Analyze parameter usage in a specific block
fn analyze_parameter_usage_in_block(
    param_var_id: VarId,
    block: &nu_protocol::ast::Block,
    context: &LintContext,
) -> ParameterUsageAnalysis {
    let mut analysis = ParameterUsageAnalysis::default();

    log::debug!(
        "Analyzing block with {} pipelines for var_id: {:?}",
        block.pipelines.len(),
        param_var_id
    );

    // Analyze each pipeline in the block
    for (i, pipeline) in block.pipelines.iter().enumerate() {
        log::debug!(
            "Analyzing block pipeline {}: {} elements",
            i,
            pipeline.elements.len()
        );
        analyze_pipeline_for_parameter_usage(pipeline, param_var_id, context, &mut analysis);
    }

    log::debug!(
        "Block analysis result: used_as_pipeline_input={}, used_with_data_commands={}, \
         used_for_data_access={}, usage_count={}",
        analysis.used_as_pipeline_input,
        analysis.used_with_data_commands,
        analysis.used_for_data_access,
        analysis.usage_count
    );

    analysis
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_pipeline_input",
        RuleCategory::Idioms,
        Severity::Warning,
        "Custom commands with single data parameters should use pipeline input for better \
         composability",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
