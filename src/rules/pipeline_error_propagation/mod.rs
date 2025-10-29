use nu_protocol::ast::{Expr, PipelineElement};

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn has_error_handling_in_pipeline(pipeline: &nu_protocol::ast::Pipeline, context: &LintContext) -> bool {
    use nu_protocol::ast::Traverse;

    let mut found_handling = Vec::new();

    // Check each element in the pipeline for error handling constructs
    for element in &pipeline.elements {
        element.expr.flat_map(
            context.working_set,
            &|expr| {
                if let Expr::Call(call) = &expr.expr {
                    let decl_name = context.working_set.get_decl(call.decl_id).name();
                    if decl_name == "try" || decl_name == "complete" {
                        return vec![true];
                    } else if decl_name == "do" {
                        // Check for -i flag in do command
                        for arg in &call.arguments {
                            if let nu_protocol::ast::Argument::Named(named) = arg
                                && named.0.item == "ignore" {
                                    return vec![true];
                                }
                        }
                    }
                }
                vec![]
            },
            &mut found_handling,
        );
    }

    !found_handling.is_empty()
}

fn has_external_command(element: &PipelineElement, context: &LintContext) -> bool {
    use nu_protocol::ast::Traverse;

    let mut found_external = Vec::new();

    element.expr.flat_map(
        context.working_set,
        &|expr| {
            if let Expr::ExternalCall(_head, _args) = &expr.expr {
                return vec![true];
            }
            vec![]
        },
        &mut found_external,
    );

    !found_external.is_empty()
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let mut violations = Vec::new();

    // Traverse the AST to find pipelines
    for pipeline in &context.ast.pipelines {
        // Skip short pipelines
        if pipeline.elements.len() < 4 {
            continue;
        }

        // Calculate pipeline span
        let pipeline_start = pipeline.elements.first().map_or(0, |e| e.expr.span.start);
        let pipeline_end = pipeline.elements.last().map_or(0, |e| e.expr.span.end);
        let pipeline_span = nu_protocol::Span::new(pipeline_start, pipeline_end);

        // Check if the pipeline has error handling
        let has_error_handling_context = has_error_handling_in_pipeline(pipeline, context);

        // Check for external commands in the pipeline
        let has_external = pipeline.elements.iter().any(|element| has_external_command(element, context));

        if has_external && !has_error_handling_context {
            violations.push(
                RuleViolation::new_static(
                    "pipeline_error_propagation",
                    "External command in pipeline without error handling - failures may be hidden",
                    pipeline_span,
                )
                .with_suggestion_static(
                    "Check external command exit codes or wrap pipeline in error handling",
                ),
            );
        }

        // Check for long pipelines without error handling
        if pipeline.elements.len() >= 4 && !has_error_handling_context {
            violations.push(
                RuleViolation::new_dynamic(
                    "pipeline_error_propagation",
                    format!("Long pipeline ({} stages) without error handling - failures may be silent", pipeline.elements.len()),
                    pipeline_span,
                )
                .with_suggestion_static(
                    "Consider: wrapping in 'try { ... }' or using 'do -i { ... }' for error handling",
                ),
            );
        }
    }

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "pipeline_error_propagation",
        RuleCategory::ErrorHandling,
        Severity::Warning,
        "Detect pipelines where errors may be lost or not properly handled",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;