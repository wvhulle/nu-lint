use nu_protocol::ast::{Expr, Expression, Pipeline};

use crate::{
    Fix, Replacement,
    ast::{call::CallExt, pipeline::PipelineExt, span::SpanExt},
    context::LintContext,
    effect::external::external_command_has_no_output,
    rule::Rule,
    violation::Violation,
};

fn command_produces_output(expr: &Expression, context: &LintContext) -> bool {
    match &expr.expr {
        Expr::ExternalCall(call, _) => {
            let cmd_name = call.span.text(context);
            !external_command_has_no_output(cmd_name)
        }
        Expr::Call(call) => {
            let output_type = context
                .working_set
                .get_decl(call.decl_id)
                .signature()
                .get_output_type();

            if output_type != nu_protocol::Type::Nothing {
                log::debug!(
                    "Command '{}' has output type: {:?}",
                    call.get_call_name(context),
                    output_type
                );
            }
            output_type != nu_protocol::Type::Nothing
        }
        _ => false,
    }
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Option<Violation> {
    let expr_before_ignore = pipeline.element_before_ignore(context)?;

    if !command_produces_output(expr_before_ignore, context) {
        return None;
    }

    let command_name = match &expr_before_ignore.expr {
        Expr::Call(call) => call.get_call_name(context),
        Expr::ExternalCall(head, _) => head.span.text(context).to_string(),
        _ => "pipeline".to_string(),
    };

    let ignore_span = pipeline.elements.last()?.expr.span;

    let elements_without_ignore = &pipeline.elements[..pipeline.elements.len() - 1];
    let start_span = elements_without_ignore.first()?.expr.span;
    let end_span = elements_without_ignore.last()?.expr.span;
    let combined_span = nu_protocol::Span::new(start_span.start, end_span.end);
    let pipeline_text = &context.source[combined_span.start..combined_span.end];

    let violation = Violation::new("Discarding command output with '| ignore'", ignore_span)
        .with_primary_label("redundant ignore")
        .with_help(format!(
            "Command '{command_name}' produces output that is being discarded with '| \
             ignore'.\n\nIf you don't need the output, consider:\n1. Removing the command if it \
             has no side effects\n2. Using error handling if you only care about \
             success/failure:\n   try {{ {command_name} }}\n3. If the output is intentionally \
             discarded, add a comment explaining why"
        ));

    let pipeline_span = nu_protocol::Span::new(
        pipeline.elements.first()?.expr.span.start,
        pipeline.elements.last()?.expr.span.end,
    );

    let fix = Fix::with_explanation(
        "Remove unnecessary '| ignore'",
        vec![Replacement::new(pipeline_span, pipeline_text.to_string())],
    );

    Some(violation.with_fix(fix))
}

fn check(context: &LintContext) -> Vec<Violation> {
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

pub const fn rule() -> Rule {
    Rule::new(
        "redundant_ignore",
        "Commands producing output that is discarded with '| ignore'",
        check,
    )
    .with_doc_url("https://www.nushell.sh/commands/docs/ignore.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
