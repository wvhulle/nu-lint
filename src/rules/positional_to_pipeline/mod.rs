use nu_protocol::{
    Span, SyntaxShape, VarId,
    ast::{Expr, Pipeline},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

pub struct FixData {
    /// Function definition span
    def_span: Span,
    /// Block containing the function body
    block_id: nu_protocol::BlockId,
    /// Parameter name being converted to pipeline input
    param_name: String,
    /// Function name
    function_name: String,
    /// Whether function is exported
    is_exported: bool,
    /// Names of remaining parameters (excluding the pipeline input parameter)
    remaining_params: Vec<String>,
}

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

/// Check if a parameter is used as the first element of any pipeline
fn parameter_used_as_pipeline_input(param_var_id: VarId, pipelines: &[Pipeline]) -> bool {
    pipelines.iter().any(|pipeline| {
        pipeline
            .elements
            .first()
            .is_some_and(|first| first.expr.matches_var(param_var_id))
    })
}

type ViolationPair = (Detection, FixData);

/// Find data parameters used as pipeline input
fn find_pipeline_data_parameters<'a>(
    signature: &'a nu_protocol::Signature,
    pipelines: &[Pipeline],
) -> Vec<&'a nu_protocol::PositionalArg> {
    signature
        .required_positional
        .iter()
        .filter(|param| {
            is_data_type_parameter(param)
                && param
                    .var_id
                    .is_some_and(|var_id| parameter_used_as_pipeline_input(var_id, pipelines))
        })
        .collect()
}

/// Analyze a function from its signature (for registered functions)
fn analyze_function_from_signature(
    signature: &nu_protocol::Signature,
    block_id: nu_protocol::BlockId,
    context: &LintContext,
) -> Vec<ViolationPair> {
    // Check if block exists in working set
    if block_id.get() >= context.working_set.num_blocks() {
        return vec![];
    }

    let block = context.working_set.get_block(block_id);
    let pipeline_params = find_pipeline_data_parameters(signature, &block.pipelines);

    pipeline_params
        .into_iter()
        .map(|param| create_violation(signature, param, block_id, context))
        .collect()
}

/// Analyze a function from AST definition (for unregistered functions)
fn analyze_function_from_ast(
    block_id: nu_protocol::BlockId,
    function_name: &str,
    context: &LintContext,
) -> Vec<ViolationPair> {
    let block = context.working_set.get_block(block_id);
    let pipeline_params = find_pipeline_data_parameters(&block.signature, &block.pipelines);

    let Some(def_span) = find_function_definition_span(function_name, context) else {
        return vec![];
    };

    // Check if function is exported by looking for 'export def' in the source
    let def_text = context.get_span_text(def_span);
    let is_exported = def_text.trim_start().starts_with("export");

    let mut function_signature = block.signature.clone();
    function_signature.name = function_name.to_string();

    pipeline_params
        .into_iter()
        .map(|param| {
            create_violation_with_span(
                &function_signature,
                param,
                def_span,
                block_id,
                is_exported,
                context,
            )
        })
        .collect()
}

fn find_function_definition_span(function_name: &str, context: &LintContext) -> Option<Span> {
    context
        .ast
        .pipelines
        .iter()
        .flat_map(|pipeline| &pipeline.elements)
        .find_map(|element| match &element.expr.expr {
            Expr::Call(call) if call.custom_command_def(context)?.name == function_name => {
                Some(call.span())
            }
            _ => None,
        })
}

fn create_violation(
    signature: &nu_protocol::Signature,
    param: &nu_protocol::PositionalArg,
    block_id: nu_protocol::BlockId,
    context: &LintContext,
) -> ViolationPair {
    let name_span = context.find_declaration_span(&signature.name);
    let def_span =
        find_function_definition_span(&signature.name, context).unwrap_or(name_span.into());

    let violation = Detection::from_file_span("Use pipeline input instead of parameter", name_span)
        .with_primary_label("function with single data parameter")
        .with_help("Pipeline input enables better composability and streaming performance");

    let remaining_params: Vec<String> = signature
        .required_positional
        .iter()
        .filter(|p| p.name != param.name)
        .map(|p| p.name.clone())
        .collect();

    let fix_data = FixData {
        def_span,
        block_id,
        param_name: param.name.clone(),
        function_name: signature.name.clone(),
        is_exported: false,
        remaining_params,
    };

    (violation, fix_data)
}

fn create_violation_with_span(
    signature: &nu_protocol::Signature,
    param: &nu_protocol::PositionalArg,
    def_span: nu_protocol::Span,
    block_id: nu_protocol::BlockId,
    is_exported: bool,
    context: &LintContext,
) -> ViolationPair {
    let name_span = context.find_declaration_span(&signature.name);

    let violation = Detection::from_file_span("Use pipeline input instead of parameter", name_span)
        .with_primary_label("function with single data parameter")
        .with_help("Pipeline input enables better composability and streaming performance");

    let remaining_params: Vec<String> = signature
        .required_positional
        .iter()
        .filter(|p| p.name != param.name)
        .map(|p| p.name.clone())
        .collect();

    let fix_data = FixData {
        def_span,
        block_id,
        param_name: param.name.clone(),
        function_name: signature.name.clone(),
        is_exported,
        remaining_params,
    };

    (violation, fix_data)
}

struct TurnPositionalIntoStreamInput;

impl DetectFix for TurnPositionalIntoStreamInput {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "turn_positional_into_stream_input"
    }

    fn explanation(&self) -> &'static str {
        "Custom commands with data parameters used as pipeline input should receive that data via \
         pipeline input ($in) instead."
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/pipelines.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
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
            .flat_map(|(block_id, decl)| {
                analyze_function_from_signature(
                    &decl.signature(),
                    nu_protocol::BlockId::new(*block_id),
                    context,
                )
            })
            .chain(
                function_definitions.iter().flat_map(|(block_id, name)| {
                    analyze_function_from_ast(*block_id, name, context)
                }),
            )
            .collect()
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        use crate::ast::string::strip_block_braces;

        let explanation = format!(
            "Use pipeline input ($in) instead of parameter (${})",
            fix_data.param_name
        );

        let block = context.working_set.get_block(fix_data.block_id);
        let body_text = context.get_span_text(block.span?);
        let body_content = strip_block_braces(body_text);

        // Transform the body: replace $param with $in, and remove "$param | " if it
        // starts a pipeline
        let param_var = format!("${}", fix_data.param_name);
        let pipeline_prefix = format!("{param_var} | ");

        let fixed_body = if body_content.trim_start().starts_with(&pipeline_prefix) {
            // Remove "$param | " from the start
            body_content
                .trim_start()
                .strip_prefix(&pipeline_prefix)
                .unwrap_or(body_content)
                .to_string()
        } else {
            // Replace all occurrences of $param with $in
            // Note: This uses string replace, but only after verifying via AST that the
            // parameter is actually used
            body_content.replace(&param_var, "$in")
        };

        let prefix = if fix_data.is_exported {
            "export def"
        } else {
            "def"
        };

        // Reconstruct parameter list from remaining parameters
        let params_str = if fix_data.remaining_params.is_empty() {
            "[]".to_string()
        } else {
            format!("[{}]", fix_data.remaining_params.join(", "))
        };

        let fixed_code = format!(
            "{} {} {} {{ {} }}",
            prefix,
            fix_data.function_name,
            params_str,
            fixed_body.trim()
        );

        Some(Fix::with_explanation(
            explanation,
            vec![Replacement::new(fix_data.def_span, fixed_code)],
        ))
    }
}

pub static RULE: &dyn Rule = &TurnPositionalIntoStreamInput;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
