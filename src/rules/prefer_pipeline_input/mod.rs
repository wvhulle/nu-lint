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
            | SyntaxShape::String
            | SyntaxShape::Any
    )
}

fn is_data_processing_command(decl_name: &str, category: &Category) -> bool {
    matches!(
        category,
        Category::Filters
            | Category::Math
            | Category::Formats
            | Category::Strings
            | Category::Conversions
            | Category::Database
    ) || matches!(
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
    fn suggests_data_operations(&self) -> bool {
        let has_data_usage = self.used_as_pipeline_input
            || self.used_with_data_commands
            || self.used_for_data_access;
        has_data_usage && !self.used_with_file_operations
    }
}

/// Helper to analyze pipelines and collect usage information
fn analyze_pipelines(
    pipelines: &[nu_protocol::ast::Pipeline],
    param_var_id: VarId,
    context: &LintContext,
) -> ParameterUsageAnalysis {
    let mut analysis = ParameterUsageAnalysis::default();

    pipelines.iter().enumerate().for_each(|(i, pipeline)| {
        log::debug!(
            "Analyzing pipeline {i}: {} elements",
            pipeline.elements.len()
        );
        analyze_pipeline_for_parameter_usage(pipeline, param_var_id, context, &mut analysis);
    });

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

/// Comprehensive analysis of how a parameter is used in a function
fn analyze_parameter_usage(param_var_id: VarId, context: &LintContext) -> ParameterUsageAnalysis {
    log::debug!("Starting analysis for parameter var_id: {param_var_id:?}");
    log::debug!("Main AST has {} pipelines", context.ast.pipelines.len());
    analyze_pipelines(&context.ast.pipelines, param_var_id, context)
}

/// Analyze a pipeline for parameter usage
fn analyze_pipeline_for_parameter_usage(
    pipeline: &nu_protocol::ast::Pipeline,
    param_var_id: VarId,
    ctx: &LintContext,
    analysis: &mut ParameterUsageAnalysis,
) {
    let Some(first_element) = pipeline.elements.first() else {
        return;
    };

    // Check if parameter is used as first element (pipeline input)
    if first_element.expr.matches_var(param_var_id) {
        analysis.used_as_pipeline_input = true;
        analysis.usage_count += 1;

        // Check if subsequent elements are data processing commands
        pipeline.elements[1..]
            .iter()
            .filter_map(|element| match &element.expr.expr {
                Expr::Call(call) => Some(call),
                _ => None,
            })
            .for_each(|call| {
                let decl = ctx.working_set.get_decl(call.decl_id);
                let sig = decl.signature();
                if is_data_processing_command(&sig.name, &sig.category) {
                    analysis.used_with_data_commands = true;
                } else if is_file_operation_command(&sig.name) {
                    analysis.used_with_file_operations = true;
                }
            });
    }

    // Also check each element for parameter usage in arguments
    pipeline
        .elements
        .iter()
        .filter_map(|element| match &element.expr.expr {
            Expr::Call(call) => Some(call),
            _ => None,
        })
        .for_each(|call| analyze_call_for_parameter_usage(call, param_var_id, ctx, analysis));
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
    if !call
        .arguments
        .iter()
        .any(|arg| argument_references_variable(arg, param_var_id))
    {
        return;
    }

    analysis.usage_count += 1;

    let sig = ctx.working_set.get_decl(call.decl_id).signature();
    if is_data_processing_command(&sig.name, &sig.category) {
        analysis.used_with_data_commands = true;
    } else if is_file_operation_command(&sig.name) {
        analysis.used_with_file_operations = true;
    } else if is_generation_command(&sig.name) {
        analysis.used_for_generation = true;
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
        | nu_protocol::ast::Argument::Spread(expr) => expr.contains_variable(var_id),
        nu_protocol::ast::Argument::Named(_) => false,
    }
}

/// Extract the function body from its declaration span for generating specific
/// suggestions
fn extract_function_body(
    decl_name: &str,
    _param_name: &str,
    context: &LintContext,
) -> Option<String> {
    context
        .ast
        .pipelines
        .iter()
        .flat_map(|pipeline| &pipeline.elements)
        .filter_map(|element| element.expr.extract_call())
        .find_map(|call| {
            let (block_id, name) = call.extract_function_definition(context)?;
            if name != decl_name {
                return None;
            }

            let block = context.working_set.get_block(block_id);
            let body_text = block.span?.text(context);
            let trimmed = body_text.trim();

            Some(
                if let Some(stripped) = trimmed.strip_prefix('{').and_then(|s| s.strip_suffix('}'))
                {
                    stripped.trim().to_string()
                } else {
                    trimmed.to_string()
                },
            )
        })
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    log::debug!("prefer_pipeline_input: Starting rule check");

    let user_functions: Vec<_> = context.new_user_functions().collect();
    log::debug!("Found {} registered user functions", user_functions.len());

    let function_definitions = context.collect_function_definitions();
    log::debug!(
        "Found {} function definitions in AST",
        function_definitions.len()
    );

    user_functions
        .iter()
        .filter_map(|(_, decl)| analyze_function_from_signature(&decl.signature(), context))
        .chain(
            function_definitions
                .iter()
                .filter_map(|(block_id, name)| analyze_function_from_ast(*block_id, name, context)),
        )
        .collect()
}

/// Common validation logic for function parameters
fn should_analyze_function<'a>(
    signature: &'a nu_protocol::Signature,
    function_name: &str,
) -> Option<&'a nu_protocol::PositionalArg> {
    // Only consider functions with exactly one required positional parameter
    if signature.required_positional.len() != 1
        || !signature.optional_positional.is_empty()
        || signature.rest_positional.is_some()
    {
        log::debug!("Skipping {function_name} - wrong parameter count");
        return None;
    }

    let param = &signature.required_positional[0];

    if !is_data_type_parameter(param) {
        log::debug!(
            "Skipping {function_name} - parameter '{}' is not a data type",
            param.name
        );
        return None;
    }

    Some(param)
}

/// Analyze a function from its signature (for registered functions)
fn analyze_function_from_signature(
    signature: &nu_protocol::Signature,
    context: &LintContext,
) -> Option<RuleViolation> {
    let param = should_analyze_function(signature, &signature.name)?;
    let param_var_id = param.var_id?;

    log::debug!(
        "Function {} parameter '{}' has var_id: {:?}",
        signature.name,
        param.name,
        param_var_id
    );

    let analysis = analyze_parameter_usage(param_var_id, context);
    analysis
        .suggests_data_operations()
        .then(|| create_violation(signature, param, context))
}

/// Analyze a function from AST definition (for unregistered functions)
fn analyze_function_from_ast(
    block_id: nu_protocol::BlockId,
    function_name: &str,
    context: &LintContext,
) -> Option<RuleViolation> {
    let block = context.working_set.get_block(block_id);
    let param = should_analyze_function(&block.signature, function_name)?;
    let param_var_id = param.var_id?;

    log::debug!(
        "Function {function_name} parameter '{}' has var_id: {param_var_id:?}",
        param.name
    );

    let analysis = analyze_parameter_usage_in_block(param_var_id, block, context);
    if !analysis.suggests_data_operations() {
        return None;
    }

    let mut function_signature = block.signature.clone();
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
    log::debug!(
        "Analyzing block with {} pipelines for var_id: {param_var_id:?}",
        block.pipelines.len()
    );
    analyze_pipelines(&block.pipelines, param_var_id, context)
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
