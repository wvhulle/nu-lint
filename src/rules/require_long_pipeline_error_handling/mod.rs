use nu_protocol::ast::Expr;

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn check_expr_for_error_handling(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Option<bool> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    let decl_name = context.working_set.get_decl(call.decl_id).name();
    if decl_name == "try" || decl_name == "complete" {
        return Some(true);
    }

    if decl_name != "do" {
        return None;
    }

    for arg in &call.arguments {
        if let nu_protocol::ast::Argument::Named(named) = arg
            && named.0.item == "ignore"
        {
            return Some(true);
        }
    }
    None
}

fn has_error_handling_in_pipeline(
    pipeline: &nu_protocol::ast::Pipeline,
    context: &LintContext,
) -> bool {
    use nu_protocol::ast::Traverse;

    let mut found_handling = Vec::new();

    for element in &pipeline.elements {
        element.expr.flat_map(
            context.working_set,
            &|expr| {
                check_expr_for_error_handling(expr, context)
                    .into_iter()
                    .collect()
            },
            &mut found_handling,
        );
    }

    !found_handling.is_empty()
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let mut violations = Vec::new();

    for pipeline in &context.ast.pipelines {
        // Only check long pipelines (4+ stages)
        if pipeline.elements.len() < 4 {
            continue;
        }

        let pipeline_start = pipeline.elements.first().map_or(0, |e| e.expr.span.start);
        let pipeline_end = pipeline.elements.last().map_or(0, |e| e.expr.span.end);
        let pipeline_span = nu_protocol::Span::new(pipeline_start, pipeline_end);

        let has_error_handling_context = has_error_handling_in_pipeline(pipeline, context);

        if !has_error_handling_context {
            violations.push(
                RuleViolation::new_dynamic(
                    "require_long_pipeline_error_handling",
                    format!(
                        "Long pipeline ({} stages) without error handling - failures may \
                         propagate silently",
                        pipeline.elements.len()
                    ),
                    pipeline_span,
                )
                .with_suggestion_static(
                    "Wrap in error handling: 'try {{ ... }}', 'do -i {{ ... }}', or add \
                     'complete' to check exit codes",
                ),
            );
        }
    }

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "require_long_pipeline_error_handling",
        RuleCategory::ErrorHandling,
        Severity::Warning,
        "Require error handling for long pipelines (4+ stages) to prevent silent failures",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
