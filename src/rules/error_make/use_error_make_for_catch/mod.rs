use std::collections::HashSet;

use nu_protocol::{
    BlockId, Span,
    ast::{Argument, Expr, Expression, Traverse},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, span::SpanExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

/// Semantic fix data: stores the call span and message expression span
pub struct FixData {
    call_span: Span,
    msg_expr_span: Option<Span>,
    is_string_literal: bool,
}

fn collect_try_blocks(context: &LintContext) -> HashSet<BlockId> {
    let mut try_block_ids = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr: &Expression| {
            let Expr::Call(call) = &expr.expr else {
                return vec![];
            };
            if call.is_call_to_command("try", context) {
                call.arguments
                    .iter()
                    .filter_map(|arg| {
                        arg.expr().and_then(|e| match &e.expr {
                            Expr::Block(block_id) => Some(*block_id),
                            _ => None,
                        })
                    })
                    .collect()
            } else {
                vec![]
            }
        },
        &mut try_block_ids,
    );
    try_block_ids.into_iter().collect()
}

struct UseErrorMakeForCatch;

impl DetectFix for UseErrorMakeForCatch {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "use_error_make_for_catch"
    }

    fn explanation(&self) -> &'static str {
        "Use 'error make' for catchable errors in functions and try blocks"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/control_flow.html#error-make")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let functions = context.collect_function_definitions();
        let try_blocks = collect_try_blocks(context);

        context.detect_with_fix_data(|expr, ctx| {
            let Expr::Call(call) = &expr.expr else {
                return vec![];
            };

            if !call.is_call_to_command("print", ctx) {
                return vec![];
            }

            let has_stderr_flag = call
                .arguments
                .iter()
                .any(|arg| matches!(arg, Argument::Named(named) if named.0.item == "stderr"));

            if !has_stderr_flag {
                return vec![];
            }

            let span = call.head;

            let in_non_main_function = span
                .find_containing_function(&functions, ctx)
                .is_some_and(|name| name != "main");

            let in_try_block = try_blocks.iter().any(|block_id| {
                ctx.working_set
                    .get_block(*block_id)
                    .span
                    .is_some_and(|s| s.contains_span(span))
            });

            if !in_non_main_function && !in_try_block {
                return vec![];
            }

            let context_hint = if in_try_block {
                "inside try block"
            } else {
                "inside reusable function"
            };

            let violation = Detection::from_global_span(
                format!("Consider 'error make' instead of 'print --stderr' {context_hint}"),
                expr.span,
            )
            .with_primary_label("stderr output in catchable context")
            .with_help(
                "Use 'error make { msg: \"...\" }' to throw a catchable exception. This allows \
                 callers to handle the error with 'try/catch'. Reserve 'print --stderr' + 'exit' \
                 for top-level unrecoverable termination in 'main'.",
            );

            // Extract message expression info for fix generation
            let msg_expr = call.get_first_positional_arg();
            let (msg_expr_span, is_string_literal) = msg_expr.map_or((None, false), |e| {
                let is_str = matches!(
                    e.expr,
                    Expr::String(_) | Expr::GlobPattern(_, _) | Expr::RawString(_)
                );
                (Some(e.span), is_str)
            });

            let fix_data = FixData {
                call_span: call.span(),
                msg_expr_span,
                is_string_literal,
            };

            vec![(violation, fix_data)]
        })
    }

    fn fix(&self, ctx: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let msg_span = fix_data.msg_expr_span?;
        let text = ctx.get_span_text(msg_span);

        let msg_text = if fix_data.is_string_literal {
            format!("\"{text}\"")
        } else {
            text.to_string()
        };

        let replacement = format!("error make {{ msg: {msg_text} }}");

        Some(Fix::with_explanation(
            "Replace with 'error make'",
            vec![Replacement::new(fix_data.call_span, replacement)],
        ))
    }
}

pub static RULE: &dyn Rule = &UseErrorMakeForCatch;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
