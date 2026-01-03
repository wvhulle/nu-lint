use nu_protocol::{Span, ast::Expr};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct ClosureParamSpacingFixData {
    whitespace_span: Span,
}

fn check_closure_param_spacing(
    context: &LintContext,
    closure_span: Span,
) -> Vec<(Detection, ClosureParamSpacingFixData)> {
    let text = context.plain_text(closure_span);

    // Validate basic structure: must start with '{' and end with '}'
    let mut chars = text.chars();
    if chars.next() != Some('{') || chars.next_back() != Some('}') {
        return vec![];
    }

    // Get inner content (between braces) using char indices for UTF-8 safety
    let inner: String = chars.collect();
    if inner.trim().is_empty() {
        return vec![];
    }

    // Find leading whitespace before the parameter pipe
    // Since we only process Expr::Closure with params, the pipe is the param
    // delimiter
    let leading_whitespace: String = inner.chars().take_while(|c| c.is_whitespace()).collect();

    if leading_whitespace.is_empty() {
        return vec![];
    }

    // Verify pipe follows the whitespace
    let after_whitespace = &inner[leading_whitespace.len()..];
    if !after_whitespace.starts_with('|') {
        return vec![];
    }

    let opening_brace_span = Span::new(closure_span.start, closure_span.start + 1);
    let whitespace_byte_len = leading_whitespace.len();
    let whitespace_span = Span::new(
        closure_span.start + 1,
        closure_span.start + 1 + whitespace_byte_len,
    );
    let pipe_span = Span::new(whitespace_span.end, whitespace_span.end + 1);

    vec![(
        Detection::from_global_span(
            "No space allowed after opening curly brace before closure parameters".to_string(),
            opening_brace_span,
        )
        .with_primary_label("opening brace")
        .with_extra_label("unwanted whitespace", whitespace_span)
        .with_extra_label(
            "parameter delimiter should follow brace directly",
            pipe_span,
        ),
        ClosureParamSpacingFixData { whitespace_span },
    )]
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

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        // Simply remove the whitespace between the opening brace and the pipe
        Some(Fix::with_explanation(
            "Remove space before closure parameters",
            vec![Replacement::new(fix_data.whitespace_span, String::new())],
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
