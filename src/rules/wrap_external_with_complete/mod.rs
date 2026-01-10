use nu_protocol::ast::{Block, Expr, Expression, Pipeline, Traverse};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    effect::{
        CommonEffect,
        external::{ExternEffect, has_external_side_effect},
    },
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn is_piped_to_complete(pipeline: &Pipeline, idx: usize, context: &LintContext) -> bool {
    pipeline.elements.get(idx + 1).is_some_and(|next| {
        matches!(&next.expr.expr, Expr::Call(call) if call.is_call_to_command("complete", context))
    })
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<Detection> {
    let mut violations = Vec::new();

    for (i, element) in pipeline.elements.iter().enumerate() {
        let Expr::ExternalCall(cmd_expr, args) = &element.expr.expr else {
            continue;
        };

        let cmd_name = cmd_expr.span_text(context);

        if !has_external_side_effect(
            cmd_name,
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            context,
            args,
        ) {
            continue;
        }

        if is_piped_to_complete(pipeline, i, context) {
            continue;
        }

        let violation = Detection::from_global_span(
            format!(
                "External command '{cmd_name}' can fail silently. Pipe to 'complete' and check \
                 'exit_code'."
            ),
            element.expr.span,
        )
        .with_primary_label("may return non-zero exit code");

        violations.push(violation);
    }

    violations
}

fn check_block(block: &Block, context: &LintContext) -> Vec<Detection> {
    let mut violations: Vec<_> = block
        .pipelines
        .iter()
        .flat_map(|p| check_pipeline(p, context))
        .collect();

    for pipeline in &block.pipelines {
        for element in &pipeline.elements {
            element.expr.flat_map(
                context.working_set,
                &|expr: &Expression| {
                    expr.extract_block_id()
                        .map(|id| check_block(context.working_set.get_block(id), context))
                        .unwrap_or_default()
                },
                &mut violations,
            );
        }
    }

    violations
}

struct WrapExternalWithComplete;

impl DetectFix for WrapExternalWithComplete {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "wrap_external_with_complete"
    }

    fn short_description(&self) -> &'static str {
        "External commands should be wrapped with 'complete' for proper error handling"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "External commands can fail and return non-zero exit codes without raising Nushell \
             errors. Unlike builtin commands, 'try' blocks do not catch external command failures \
             based on exit code. Use '| complete' to capture stdout, stderr, and exit_code in a \
             record, then check the exit_code field to handle errors properly. Without this, \
             failures may go unnoticed and cause silent data corruption or unexpected behavior \
             downstream.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/complete.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(check_block(context.ast, context))
    }
}

pub static RULE: &dyn Rule = &WrapExternalWithComplete;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
