use nu_protocol::{Span, ast::Expr};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct ClosureParamSpacingFixData {
    span: Span,
}

fn check_closure_param_spacing(
    context: &LintContext,
    span: Span,
) -> Vec<(Detection, ClosureParamSpacingFixData)> {
    let text = context.get_span_text(span);
    if text.is_empty() || !text.starts_with('{') || !text.ends_with('}') {
        return vec![];
    }
    let inner = &text[1..text.len() - 1];
    if inner.trim().is_empty() {
        return vec![];
    }

    // Check for space before pipe (closure params)
    if let Some(pipe_pos) = inner.find('|') {
        if pipe_pos > 0 && inner[..pipe_pos].chars().all(char::is_whitespace) {
            let opening_brace_span = Span::new(span.start, span.start + 1);
            return vec![(
                Detection::from_global_span(
                    "No space allowed after opening curly brace before closure parameters"
                        .to_string(),
                    opening_brace_span,
                )
                .with_primary_label("opening curly brace")
                .with_extra_span(Span::new(span.start + 1, span.start + 1 + pipe_pos))
                .with_help("Use {|param| instead of { |param|"),
                ClosureParamSpacingFixData { span },
            )];
        }
    }
    vec![]
}

fn has_block_params(context: &LintContext, block_id: nu_protocol::BlockId) -> bool {
    let block = context.working_set.get_block(block_id);
    !block.signature.required_positional.is_empty()
        || !block.signature.optional_positional.is_empty()
        || block.signature.rest_positional.is_some()
}

struct ClosureParamSpacing;

impl DetectFix for ClosureParamSpacing {
    type FixInput<'a> = ClosureParamSpacingFixData;

    fn id(&self) -> &'static str {
        "curly_closure_param_spacing"
    }

    fn explanation(&self) -> &'static str {
        "No space allowed between opening curly brace and closure parameters"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/style_guide.html#one-line-format")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| match &expr.expr {
            Expr::Closure(block_id) if has_block_params(ctx, *block_id) => {
                check_closure_param_spacing(ctx, expr.span)
            }
            _ => vec![],
        })
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let text = context.get_span_text(fix_data.span);
        let inner = &text[1..text.len() - 1];
        let trimmed = inner.trim_start();
        let fixed = format!("{{{trimmed}}}");

        Some(Fix::with_explanation(
            "Remove space before closure parameters",
            vec![Replacement::new(fix_data.span, fixed)],
        ))
    }
}

pub static RULE: &dyn Rule = &ClosureParamSpacing;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
