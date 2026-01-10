use nu_protocol::{
    Type,
    ast::{Block, Expr, Pipeline, PipelineElement, Traverse},
};

use crate::{ast::expression::ExpressionExt, context::LintContext, violation::Detection};

/// Configuration for checking a specific type of external tool
pub struct ConversionSpec<'a> {
    /// Function to check if a command name matches this tool category
    pub matches_command: &'a dyn Fn(&str) -> bool,
    /// Function to check if a type needs conversion for this tool
    pub matches_type: &'a dyn Fn(&Type) -> bool,
}

/// Check all pipelines in the AST for violations
pub fn check_all_pipelines<FixData>(
    context: &LintContext,
    spec: &ConversionSpec,
    create_violation: impl Fn(&Type, &str, &PipelineElement, &PipelineElement) -> (Detection, FixData)
    + Copy,
) -> Vec<(Detection, FixData)> {
    let mut violations = Vec::new();
    check_block_recursive(
        context.ast,
        context,
        spec,
        create_violation,
        &mut violations,
    );
    violations
}

/// Recursively check a block and all nested blocks
fn check_block_recursive<FixData>(
    block: &Block,
    context: &LintContext,
    spec: &ConversionSpec,
    create_violation: impl Fn(&Type, &str, &PipelineElement, &PipelineElement) -> (Detection, FixData)
    + Copy,
    violations: &mut Vec<(Detection, FixData)>,
) {
    // Check all pipelines in this block
    for pipeline in &block.pipelines {
        violations.extend(check_pipeline(pipeline, context, spec, create_violation));
    }

    // Find and recursively check all nested blocks
    let mut nested_block_ids = Vec::new();
    for pipeline in &block.pipelines {
        for element in &pipeline.elements {
            element.expr.flat_map(
                context.working_set,
                &|expr| match &expr.expr {
                    Expr::Block(id)
                    | Expr::RowCondition(id)
                    | Expr::Closure(id)
                    | Expr::Subexpression(id) => vec![*id],
                    _ => vec![],
                },
                &mut nested_block_ids,
            );
        }
    }

    for &block_id in &nested_block_ids {
        check_block_recursive(
            context.working_set.get_block(block_id),
            context,
            spec,
            create_violation,
            violations,
        );
    }
}

/// Check a single pipeline for violations
fn check_pipeline<FixData>(
    pipeline: &Pipeline,
    context: &LintContext,
    spec: &ConversionSpec,
    create_violation: impl Fn(&Type, &str, &PipelineElement, &PipelineElement) -> (Detection, FixData),
) -> Vec<(Detection, FixData)> {
    pipeline
        .elements
        .windows(2)
        .filter_map(|window| {
            check_pipeline_pair(&window[0], &window[1], context, spec, &create_violation)
        })
        .collect()
}

/// Check if a pipeline pair has the violation pattern
fn check_pipeline_pair<FixData>(
    left: &PipelineElement,
    right: &PipelineElement,
    context: &LintContext,
    spec: &ConversionSpec,
    create_violation: &impl Fn(&Type, &str, &PipelineElement, &PipelineElement) -> (Detection, FixData),
) -> Option<(Detection, FixData)> {
    // Check if right is an external call
    let Expr::ExternalCall(head, _args) = &right.expr.expr else {
        return None;
    };

    // Get command name and check if it matches
    let cmd_name = context.plain_text(head.span);
    let clean_name = cmd_name.trim_start_matches('^');

    if !(spec.matches_command)(clean_name) {
        return None;
    }

    // Infer the type being piped in
    let input_type = left.expr.infer_output_type(context)?;

    // Check if the type needs conversion
    (spec.matches_type)(&input_type).then(|| create_violation(&input_type, cmd_name, left, right))
}
