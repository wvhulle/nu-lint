use std::collections::HashSet;

use nu_lint_analysis::{
    LintLevel,
    ast::{call::CallExt, span::SpanExt, string::StringFormat},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::{
    BlockId, Span,
    ast::{Argument, Expr, Expression, Traverse},
};

/// Semantic fix data: stores the call span and message expression span
pub struct FixData {
    call_span: Span,
    message_span: Option<Span>,
    is_bare_word: bool,
}

fn collect_try_blocks(context: &LintContext) -> HashSet<BlockId> {
    let mut try_block_ids = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr: &Expression| {
            let Expr::Call(call) = &expr.expr else {
                return vec![];
            };
            if call.builtin_name(context) == Some("try") {
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
        "error_make_for_non_fatal"
    }

    fn short_description(&self) -> &'static str {
        "Use `error make` for catchable errors in functions and try blocks"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/control_flow.html#error-make")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let functions = context.custom_commands();
        let try_blocks = collect_try_blocks(context);

        context.detect_with_fix_data(|expr, ctx| {
            let Expr::Call(call) = &expr.expr else {
                return vec![];
            };

            if call.builtin_name(ctx) != Some("print") {
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
                .is_some_and(|def| !def.is_main());

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
            .with_primary_label("stderr output in catchable context");

            // Extract message expression info for fix generation
            let message_expr = call.get_first_positional_arg();
            let (message_span, is_bare_word) =
                message_expr.map_or((None, false), |message_operand| {
                    let written_bare = matches!(message_operand.expr, Expr::GlobPattern(..))
                        || matches!(
                            StringFormat::from_expression(message_operand, ctx),
                            Some(StringFormat::BareWord(_))
                        );
                    (Some(message_operand.span), written_bare)
                });

            let fix_data = FixData {
                call_span: call.span(),
                message_span,
                is_bare_word,
            };

            vec![(violation, fix_data)]
        })
    }

    fn fix(&self, ctx: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let msg_span = fix_data.message_span?;
        let text = ctx.span_text(msg_span);

        let msg_text = if fix_data.is_bare_word {
            format!("\"{text}\"")
        } else {
            text.to_string()
        };

        let replacement = format!("error make {{ msg: {msg_text} }}");

        Some(Fix {
            explanation: "Replace with 'error make'".into(),
            replacements: vec![Replacement::new(fix_data.call_span, replacement)],
        })
    }
}

pub static RULE: &dyn Rule = &UseErrorMakeForCatch;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
