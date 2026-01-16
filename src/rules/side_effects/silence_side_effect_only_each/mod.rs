use nu_protocol::{
    Span,
    ast::{Block, Expr, Expression},
};

use crate::{
    Fix, LintLevel, Replacement,
    ast::{block::BlockExt, call::CallExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

pub struct FixData {
    replace_span: Span,
    list_span: Span,
    param_name: String,
    body_span: Span,
    pipeline_elements_before_each: usize,
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

fn extract_pipeline_info(expr: &Expression, ctx: &LintContext) -> Option<(Span, usize)> {
    for pipeline in &ctx.ast.pipelines {
        for (i, elem) in pipeline.elements.iter().enumerate() {
            if elem.expr.span.contains_span(expr.span) && i > 0 {
                let first = &pipeline.elements[0];
                let last_before = &pipeline.elements[i - 1];
                let span = Span::new(first.expr.span.start, last_before.expr.span.end);
                return Some((span, i));
            }
        }
    }
    None
}

fn get_closure_body_span(block: &Block) -> Option<Span> {
    if block.pipelines.is_empty() {
        return None;
    }

    let first_pipeline = block.pipelines.first()?;
    let last_pipeline = block.pipelines.last()?;

    let first_elem = first_pipeline.elements.first()?;
    let last_elem = last_pipeline.elements.last()?;

    Some(Span::new(
        first_elem.expr.span.start,
        last_elem.expr.span.end,
    ))
}

struct UseForOverEach;

impl DetectFix for UseForOverEach {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "silence_side_effect_only_each"
    }

    fn short_description(&self) -> &'static str {
        "Silence loops that only have side effects and no pipeline output."
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/loops.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
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

                let param_name = block
                    .signature
                    .required_positional
                    .first()
                    .map_or_else(|| "item".to_string(), |p| p.name.clone());

                let Some(body_span) = get_closure_body_span(block) else {
                    return vec![];
                };

                let (list_span, pipeline_elements_before_each) = extract_pipeline_info(expr, ctx)
                    .unwrap_or_else(|| (Span::new(call.span().start, call.span().start), 0));

                let violation = Detection::from_global_span(
                    "Consider using a 'for' loop for side effects only",
                    call.span(),
                )
                .with_primary_label("closure returns nothing");

                let fix_data = FixData {
                    replace_span: call.span(),
                    list_span,
                    param_name,
                    body_span,
                    pipeline_elements_before_each,
                };

                vec![(violation, fix_data)]
            }
            _ => vec![],
        })
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        // Only provide auto-fix when clean conversion to for loop is possible
        // (single pipeline element before each). Multi-stage pipelines are more
        // complex and require manual refactoring.
        if fix_data.pipeline_elements_before_each != 1 {
            return None;
        }

        let list = if fix_data.list_span.is_empty() {
            return None;
        } else {
            context.span_text(fix_data.list_span).trim()
        };

        let body = context.span_text(fix_data.body_span).trim();
        let fix_text = format!("for {} in {} {{ {} }}", fix_data.param_name, list, body);

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
