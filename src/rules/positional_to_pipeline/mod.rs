use nu_protocol::{
    Span, SyntaxShape, VarId,
    ast::{Expr, Pipeline},
};

use crate::{
    LintLevel,
    ast::{
        block::BlockExt, call::CallExt, declaration::CustomCommandDef, expression::ExpressionExt,
    },
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
    /// `VarId` of the parameter being converted
    param_var_id: VarId,
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
    def: &CustomCommandDef,
    context: &LintContext,
) -> Vec<ViolationPair> {
    // Check if block exists in working set
    if def.body.get() >= context.working_set.num_blocks() {
        return vec![];
    }

    let block = context.working_set.get_block(def.body);
    let pipeline_params = find_pipeline_data_parameters(&def.signature, &block.pipelines);

    pipeline_params
        .into_iter()
        .filter_map(|param| create_violation(def, param, context))
        .collect()
}

/// Analyze a function from AST definition (for unregistered functions)
fn analyze_function_from_ast(def: &CustomCommandDef, context: &LintContext) -> Vec<ViolationPair> {
    let block = context.working_set.get_block(def.body);
    let pipeline_params = find_pipeline_data_parameters(&block.signature, &block.pipelines);

    let Some(def_span) = find_function_definition_span(&def.name, context) else {
        return vec![];
    };

    let mut function_signature = block.signature.clone();
    function_signature.name.clone_from(&def.name);

    pipeline_params
        .into_iter()
        .filter_map(|param| {
            create_violation_with_span(
                &function_signature,
                param,
                def_span,
                def.body,
                def.is_exported(),
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
    def: &CustomCommandDef,
    param: &nu_protocol::PositionalArg,
    context: &LintContext,
) -> Option<ViolationPair> {
    let name_span = def.declaration_span(context);
    let signature = &def.signature;
    let block_id = def.body;
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
        param_var_id: param.var_id?,
        remaining_params,
    };

    Some((violation, fix_data))
}

fn create_violation_with_span(
    signature: &nu_protocol::Signature,
    param: &nu_protocol::PositionalArg,
    def_span: nu_protocol::Span,
    block_id: nu_protocol::BlockId,
    _is_exported: bool,
) -> Option<ViolationPair> {
    let violation =
        Detection::from_global_span("Use pipeline input instead of parameter", def_span)
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
        param_var_id: param.var_id?,
        remaining_params,
    };

    Some((violation, fix_data))
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

        let function_definitions = context.custom_commands();
        log::debug!(
            "Found {} function definitions in AST",
            function_definitions.len()
        );

        function_definitions
            .iter()
            .flat_map(|def| {
                analyze_function_from_signature(def, context)
                    .into_iter()
                    .chain(analyze_function_from_ast(def, context))
            })
            .collect()
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let explanation = format!(
            "Use pipeline input ($in) instead of parameter (${})",
            fix_data.param_name
        );

        let block = context.working_set.get_block(fix_data.block_id);

        // Find all usages of the parameter variable in the block
        let mut var_spans =
            block.find_var_usage_spans(fix_data.param_var_id, context, |_, _, _| true);

        if var_spans.is_empty() {
            return None;
        }

        // Sort spans by start position
        var_spans.sort_by_key(|span| span.start);
        var_spans.dedup();

        // Filter out overlapping spans (keep the largest span for each start position)
        let filtered_spans: Vec<Span> =
            var_spans.iter().copied().fold(Vec::new(), |mut acc, span| {
                if let Some(last) = acc.last_mut() {
                    // If this span starts at the same position, keep the larger one
                    if span.start == last.start {
                        if span.end > last.end {
                            *last = span;
                        }
                    } else if span.start >= last.end {
                        // Non-overlapping span
                        acc.push(span);
                    }
                    // Skip spans that start within the previous span
                } else {
                    acc.push(span);
                }
                acc
            });

        // Check if the first usage is at the start of a pipeline (i.e., "$param | ...")
        let first_span = filtered_spans.first()?;
        let first_is_pipeline_start = context
            .get_span_text(*first_span)
            .trim()
            .starts_with(&format!("${}", fix_data.param_name));

        // Check if there's a " | " after the first usage that we should remove
        let after_first_end = first_span.end;
        let def_span_end = fix_data.def_span.end;
        let pipeline_separator = " | ";

        let remove_pipeline_prefix = if first_is_pipeline_start
            && after_first_end + 3 <= def_span_end
        {
            // Check the text after the first var usage
            let after_span = Span::new(after_first_end, (after_first_end + 3).min(def_span_end));
            let after_text = context.get_span_text(after_span);
            after_text == pipeline_separator
        } else {
            false
        };

        let mut replacements = Vec::new();

        // Build replacements for variable usages
        for (i, &span) in filtered_spans.iter().enumerate() {
            if i == 0 && remove_pipeline_prefix {
                // For first usage followed by " | ", remove both the var and the separator
                let extended_span = Span::new(span.start, span.end + 3);
                replacements.push(Replacement::new(extended_span, String::new()));
            } else {
                // Replace $param with $in
                replacements.push(Replacement::new(span, "$in".to_string()));
            }
        }

        // We also need to update the function signature to remove the parameter
        // Find the parameter list span and rebuild it
        let def_text = context.get_span_text(fix_data.def_span);

        // Find the opening [ after the function name
        let bracket_start_idx = def_text.find('[')?;
        let bracket_end_idx = def_text.find(']')?;

        let bracket_start = fix_data.def_span.start + bracket_start_idx;
        let bracket_end = fix_data.def_span.start + bracket_end_idx + 1;
        let param_list_span = Span::new(bracket_start, bracket_end);

        // Reconstruct parameter list from remaining parameters
        let new_params_str = if fix_data.remaining_params.is_empty() {
            "[]".to_string()
        } else {
            format!("[{}]", fix_data.remaining_params.join(", "))
        };

        replacements.push(Replacement::new(param_list_span, new_params_str));

        Some(Fix::with_explanation(explanation, replacements))
    }
}

pub static RULE: &dyn Rule = &TurnPositionalIntoStreamInput;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
