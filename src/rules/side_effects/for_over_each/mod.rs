use nu_protocol::{
    Span,
    ast::{Expr, Expression},
};

use crate::{
    Fix, LintLevel, Replacement,
    ast::{block::BlockExt, call::CallExt, span::SpanExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

pub struct FixData {
    replace_span: Span,
    list_span: Span,
    param_name: String,
    body_span: Span,
}

/// Checks if an expression returns Nothing (only side effects, no data)
fn returns_nothing(expr: &Expression, ctx: &LintContext) -> bool {
    match &expr.expr {
        Expr::Call(call) => {
            let decl = ctx.working_set.get_decl(call.decl_id);
            let sig = decl.signature();

            // If all output types are Nothing, then this returns nothing
            sig.input_output_types
                .iter()
                .all(|(_in, out)| matches!(out, nu_protocol::Type::Nothing))
        }
        _ => false,
    }
}

/// Checks if a block contains only side effects (no return values used)
fn block_has_only_side_effects(block_id: nu_protocol::BlockId, ctx: &LintContext) -> bool {
    let block = ctx.working_set.get_block(block_id);

    // Use the block's inferred output type
    let output_type = block.infer_output_type(ctx);

    // If the block returns Nothing, it's side-effect-only
    if matches!(output_type, nu_protocol::Type::Nothing) {
        return true;
    }

    // For Type::Any, check the last pipeline element
    if matches!(output_type, nu_protocol::Type::Any) {
        let Some(last_pipeline) = block.pipelines.last() else {
            return false;
        };
        let Some(last_element) = last_pipeline.elements.last() else {
            return false;
        };

        return returns_nothing(&last_element.expr, ctx);
    }

    false
}

fn extract_pipeline_before_each(expr: &Expression, ctx: &LintContext) -> Option<Span> {
    // Find the pipeline that contains the each call
    for pipeline in &ctx.ast.pipelines {
        for (i, elem) in pipeline.elements.iter().enumerate() {
            if elem.expr.span.contains_span(expr.span) && i > 0 {
                // Get all elements before the each call
                let first = &pipeline.elements[0];
                let last_before = &pipeline.elements[i - 1];
                return Some(Span::new(first.expr.span.start, last_before.expr.span.end));
            }
        }
    }
    None
}

struct UseForOverEach;

impl DetectFix for UseForOverEach {
    type FixInput = FixData;

    fn id(&self) -> &'static str {
        "for_over_each_side_effects"
    }

    fn explanation(&self) -> &'static str {
        "Use 'for' loop instead of 'each' for side effects only"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/loops.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
        context.detect_with_fix_data(|expr, ctx| match &expr.expr {
            Expr::Call(call) if call.is_call_to_command("each", ctx) => {
                let Some(closure_arg) = call.get_first_positional_arg() else {
                    return vec![];
                };

                let Expr::Closure(block_id) = &closure_arg.expr else {
                    return vec![];
                };

                if !block_has_only_side_effects(*block_id, ctx) {
                    return vec![];
                }

                let block = ctx.working_set.get_block(*block_id);

                // Extract parameter name from closure signature
                let param_name = block
                    .signature
                    .required_positional
                    .first()
                    .map_or_else(|| "item".to_string(), |p| p.name.clone());

                // Get the body span from the block - it should always exist
                let Some(body_span) = block.span else {
                    return vec![];
                };

                // Try to find the list being piped to each
                let list_span = extract_pipeline_before_each(expr, ctx)
                    .unwrap_or_else(|| Span::new(call.span().start, call.span().start));

                let violation = Detection::from_global_span(
                    "Use 'for' loop instead of 'each' for side effects only",
                    call.span(),
                )
                .with_primary_label("closure returns nothing")
                .with_help(
                    "Each iteration returns nothing, producing an empty table. Use 'for' loop or \
                     pipe to 'ignore'",
                );

                let fix_data = FixData {
                    replace_span: call.span(),
                    list_span,
                    param_name,
                    body_span,
                };

                vec![(violation, fix_data)]
            }
            _ => vec![],
        })
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput) -> Option<Fix> {
        let list = if fix_data.list_span.is_empty() {
            return None;
        } else {
            fix_data.list_span.source_code(context).trim()
        };

        let body = fix_data.body_span.source_code(context).trim();

        let fix_text = format!("for {} in {} {}", fix_data.param_name, list, body);

        Some(Fix::with_explanation(
            "Convert each to for loop",
            vec![Replacement::new(fix_data.replace_span, fix_text)],
        ))
    }
}

pub static RULE: &dyn Rule = &UseForOverEach;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
