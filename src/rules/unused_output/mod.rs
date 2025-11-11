use nu_protocol::ast::{Expr, Expression, Pipeline};

use crate::{
    ast::{call::CallExt, ext_command::ExternalCommandExt, pipeline::PipelineExt, span::SpanExt},
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

/// Check if a command produces output based on its signature's output type
/// Falls back to a whitelist for commands with `Type::Any`
fn command_produces_output(expr: &Expression, context: &LintContext) -> bool {
    match &expr.expr {
        Expr::ExternalCall(call, _) => {
            let cmd_name = call.span.text(context);

            cmd_name.is_known_external_output_command()
                || !cmd_name.is_known_external_no_output_command()
        }
        Expr::Call(call) => {
            let cmd_name = call.get_call_name(context);

            let decl = context.working_set.get_decl(call.decl_id);
            let signature = decl.signature();

            // Check the output type from the signature
            let output_type = signature.get_output_type();

            if output_type == nu_protocol::Type::Nothing {
                // Definitely produces no output
                false
            } else {
                // Has a specific output type (String, List, etc.) - produces output
                log::debug!("Command '{cmd_name}' has output type: {output_type:?}");
                true
            }
        }
        _ => false,
    }
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Option<RuleViolation> {
    let prev_expr = pipeline.element_before_ignore(context)?;

    if !command_produces_output(prev_expr, context) {
        return None;
    }

    let prev_call = match &prev_expr.expr {
        Expr::Call(call) => call.get_call_name(context),
        _ => "pipeline".to_string(),
    };

    let ignore_span = pipeline.elements.last()?.expr.span;

    Some(
        RuleViolation::new_static(
            "unused_output",
            "Discarding command output with '| ignore'",
            ignore_span,
        )
        .with_suggestion_dynamic(format!(
            "Command '{prev_call}' produces output that is being discarded with '| ignore'.\n\nIf \
             you don't need the output, consider:\n1. Removing the command if it has no side \
             effects\n2. Using error handling if you only care about success/failure:\n   try {{ \
             {prev_call} }}\n3. If the output is intentionally discarded, add a comment \
             explaining why"
        )),
    )
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context
        .ast
        .pipelines
        .iter()
        .filter_map(|pipeline| check_pipeline(pipeline, context))
        .chain(context.collect_rule_violations(|expr, ctx| {
            match &expr.expr {
                Expr::Block(block_id) | Expr::Closure(block_id) | Expr::Subexpression(block_id) => {
                    ctx.working_set
                        .get_block(*block_id)
                        .pipelines
                        .iter()
                        .filter_map(|pipeline| check_pipeline(pipeline, ctx))
                        .collect()
                }
                _ => vec![],
            }
        }))
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "unused_output",
        RuleCategory::Idioms,
        Severity::Warning,
        "Commands producing output that is discarded with '| ignore'",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
