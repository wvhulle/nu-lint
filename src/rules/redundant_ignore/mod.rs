use nu_protocol::ast::{Expr, Expression, Pipeline};

use crate::{
    Fix, LintLevel, Replacement,
    ast::{call::CallExt, pipeline::PipelineExt, span::SpanExt},
    context::LintContext,
    effect::external::external_command_has_no_output,
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct RedundantIgnoreFixData {
    pipeline_span: nu_protocol::Span,
    pipeline_text: String,
}

fn command_produces_output(expr: &Expression, context: &LintContext) -> bool {
    match &expr.expr {
        Expr::ExternalCall(call, _) => {
            let cmd_name = call.span.source_code(context);
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

fn check_pipeline(
    pipeline: &Pipeline,
    context: &LintContext,
) -> Option<(Detection, RedundantIgnoreFixData)> {
    let expr_before_ignore = pipeline.element_before_ignore(context)?;

    if !command_produces_output(expr_before_ignore, context) {
        return None;
    }

    let command_name = match &expr_before_ignore.expr {
        Expr::Call(call) => call.get_call_name(context),
        Expr::ExternalCall(head, _) => head.span.source_code(context).to_string(),
        _ => "pipeline".to_string(),
    };

    let ignore_span = pipeline.elements.last()?.expr.span;

    let elements_without_ignore = &pipeline.elements[..pipeline.elements.len() - 1];
    let start_span = elements_without_ignore.first()?.expr.span;
    let end_span = elements_without_ignore.last()?.expr.span;
    let combined_span = nu_protocol::Span::new(start_span.start, end_span.end);
    let pipeline_text = combined_span.source_code(context);

    let violation =
        Detection::from_global_span("Discarding command output with '| ignore'", ignore_span)
            .with_primary_label("redundant ignore")
            .with_help(format!(
                "Command '{command_name}' produces output that is being discarded with '| \
                 ignore'.\n\nIf you don't need the output, consider:\n1. Removing the command if \
                 it has no side effects\n2. Using error handling if you only care about \
                 success/failure:\n   try {{ {command_name} }}\n3. If the output is intentionally \
                 discarded, add a comment explaining why"
            ));

    let pipeline_span = nu_protocol::Span::new(
        pipeline.elements.first()?.expr.span.start,
        pipeline.elements.last()?.expr.span.end,
    );

    let fix_data = RedundantIgnoreFixData {
        pipeline_span,
        pipeline_text: pipeline_text.to_string(),
    };

    Some((violation, fix_data))
}

struct RedundantIgnore;

impl DetectFix for RedundantIgnore {
    type FixInput<'a> = RedundantIgnoreFixData;

    fn id(&self) -> &'static str {
        "redundant_ignore"
    }

    fn explanation(&self) -> &'static str {
        "Commands producing output that is discarded with '| ignore'"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/ignore.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut violations: Vec<_> = context
            .ast
            .pipelines
            .iter()
            .filter_map(|pipeline| check_pipeline(pipeline, context))
            .collect();

        violations.extend(context.detect_with_fix_data(|expr, ctx| {
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
        }));

        violations
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix::with_explanation(
            "Remove unnecessary '| ignore'",
            vec![Replacement::new(
                fix_data.pipeline_span,
                fix_data.pipeline_text.clone(),
            )],
        ))
    }
}

pub static RULE: &dyn Rule = &RedundantIgnore;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
