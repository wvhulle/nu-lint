use nu_lint_analysis::{
    LintLevel,
    ast::{
        block::BlockExt,
        call::CallExt,
        declaration::{CustomCommandDef, parameter_removal_span},
        expression::ExpressionExt,
    },
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::{
    Span, SyntaxShape, VarId,
    ast::{Expr, Pipeline, Traverse},
};

pub struct FixData {
    signature_span: Span,
    signature_is_annotated: bool,
    /// Block containing the function body
    block_id: nu_protocol::BlockId,
    /// `VarId` of the parameter being converted
    param_var_id: VarId,
}

fn command_has_call_sites(body_block_id: nu_protocol::BlockId, context: &LintContext) -> bool {
    let mut call_site_spans: Vec<Span> = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| match &expr.expr {
            Expr::Call(call)
                if context.working_set.get_decl(call.decl_id).block_id() == Some(body_block_id) =>
            {
                vec![expr.span]
            }
            _ => vec![],
        },
        &mut call_site_spans,
    );
    !call_site_spans.is_empty()
}

/// Check if a parameter is a data type that would benefit from pipeline input
fn is_data_type_parameter(param: &nu_protocol::PositionalArg) -> bool {
    log::trace!("Parameter '{}' has shape: {:?}", param.name, param.shape);
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

    pipeline_params
        .into_iter()
        .filter_map(|param| create_violation_with_span(param, def_span, def))
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
    let block_id = def.body;

    let violation = Detection::from_file_span("Use pipeline input instead of parameter", name_span)
        .with_primary_label("function with single data parameter");

    let fix_data = FixData {
        signature_span: def.signature_span,
        signature_is_annotated: !def.signature.input_output_types.is_empty(),
        block_id,
        param_var_id: param.var_id?,
    };

    Some((violation, fix_data))
}

fn create_violation_with_span(
    param: &nu_protocol::PositionalArg,
    def_span: nu_protocol::Span,
    def: &CustomCommandDef,
) -> Option<ViolationPair> {
    let violation =
        Detection::from_global_span("Use pipeline input instead of parameter", def_span)
            .with_primary_label("function with single data parameter");

    let fix_data = FixData {
        signature_span: def.signature_span,
        signature_is_annotated: !def.signature.input_output_types.is_empty(),
        block_id: def.body,
        param_var_id: param.var_id?,
    };

    Some((violation, fix_data))
}

struct TurnPositionalIntoStreamInput;

impl DetectFix for TurnPositionalIntoStreamInput {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "positional_to_pipeline"
    }

    fn short_description(&self) -> &'static str {
        "Data parameter convertible to pipeline input"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/pipelines.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let function_definitions = context.custom_commands();

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
        let block = context.working_set.get_block(fix_data.block_id);
        let required_params = &block.signature.required_positional;
        let param_name = &required_params
            .iter()
            .find(|param| param.var_id == Some(fix_data.param_var_id))?
            .name;
        let explanation = format!("Use pipeline input ($in) instead of parameter (${param_name})");

        // Find all usages of the parameter variable in the block
        let mut var_spans = block.var_usages(fix_data.param_var_id, context);

        if var_spans.is_empty() {
            return None;
        }

        // Sort spans by start position
        var_spans.sort_by_key(|span| span.start);
        var_spans.dedup();

        if command_has_call_sites(fix_data.block_id, context) || fix_data.signature_is_annotated {
            return None;
        }

        let is_top_level_pipeline_head = |usage_span: &Span| {
            block.pipelines.iter().any(|pipeline| {
                pipeline
                    .elements
                    .first()
                    .is_some_and(|head| head.expr.span.start == usage_span.start)
            })
        };
        if !var_spans.iter().all(is_top_level_pipeline_head) {
            return None;
        }

        let first_span = *var_spans.first()?;
        let next_element_start = block.pipelines.iter().find_map(|pipeline| {
            let (head, next_element) = (pipeline.elements.first()?, pipeline.elements.get(1)?);
            (head.expr.span == first_span).then_some(next_element.expr.span.start)
        });

        let mut replacements: Vec<Replacement> = var_spans
            .iter()
            .map(|&span| match next_element_start {
                Some(next_start) if span == first_span => {
                    Replacement::new(Span::new(span.start, next_start), String::new())
                }
                _ => Replacement::new(span, "$in".to_string()),
            })
            .collect();

        let parameter_name_span = context
            .working_set
            .get_variable(fix_data.param_var_id)
            .declaration_span;
        replacements.push(Replacement::new(
            parameter_removal_span(parameter_name_span, fix_data.signature_span, context)?,
            String::new(),
        ));

        Some(Fix {
            explanation: explanation.into(),
            replacements,
        })
    }
}

pub static RULE: &dyn Rule = &TurnPositionalIntoStreamInput;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
