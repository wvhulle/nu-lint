use nu_protocol::{BlockId, Span, ast::Expr};

use super::{has_explicit_pipe_delimiters, is_record_type};
use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

/// Get the span of the first expression in a block's pipelines.
fn first_body_expr_span(context: &LintContext, block_id: BlockId) -> Option<Span> {
    context
        .working_set
        .get_block(block_id)
        .pipelines
        .first()?
        .elements
        .first()
        .map(|elem| elem.expr.span)
}

/// Find the closing pipe `|` char index in closure text like `{|x| body}`.
fn closing_pipe_char_index(text: &str) -> Option<usize> {
    let mut chars = text.chars().enumerate().peekable();
    chars.next().filter(|(_, c)| *c == '{')?;
    while chars.peek().is_some_and(|(_, c)| c.is_whitespace()) {
        chars.next();
    }
    chars.next().filter(|(_, c)| *c == '|')?;
    chars.find(|(_, c)| *c == '|').map(|(i, _)| i)
}

/// Check spacing around closure body
fn check_body_spacing(text: &str, pipe_idx: usize) -> (bool, bool) {
    let chars: Vec<char> = text.chars().collect();
    let after_pipe = chars.get(pipe_idx + 1).is_some_and(|c| !c.is_whitespace());
    let before_brace = chars
        .get(chars.len().saturating_sub(2))
        .is_some_and(|c| !c.is_whitespace());
    (after_pipe, before_brace)
}

/// Format closure with proper spacing.
fn format_closure(text: &str, pipe_idx: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    let params: String = chars[..=pipe_idx].iter().collect();
    let body: String = chars[pipe_idx + 1..chars.len() - 1].iter().collect();
    format!("{params} {} }}", body.trim())
}

fn check_closure(
    ctx: &LintContext,
    closure_span: Span,
    block_id: BlockId,
) -> Option<(Detection, Span)> {
    let text = ctx.span_text(closure_span);
    let pipe_idx = closing_pipe_char_index(text)?;
    let (needs_after, needs_before) = check_body_spacing(text, pipe_idx);

    if !needs_after && !needs_before {
        return None;
    }

    let body_span = first_body_expr_span(ctx, block_id);
    let mut detection = Detection::from_global_span(
        "Closure body needs space after `|` and before `}`: `{|x| body }`".to_string(),
        closure_span,
    );

    if needs_after {
        let span = body_span.map_or_else(
            || Span::new(closure_span.end - 2, closure_span.end - 1),
            |s| Span::new(s.start.saturating_sub(1), s.start),
        );
        detection = detection.with_extra_label("add space after `|`", span);
    }
    if needs_before {
        let span = body_span.map_or_else(
            || Span::new(closure_span.end - 1, closure_span.end),
            |s| Span::new(s.end, s.end + 1),
        );
        detection = detection.with_extra_label("add space before `}`", span);
    }

    Some((detection, closure_span))
}

struct ClosureBodySpacing;

impl DetectFix for ClosureBodySpacing {
    type FixInput<'a> = Span;

    fn id(&self) -> &'static str {
        "closure_pipe_body_spacing"
    }

    fn short_description(&self) -> &'static str {
        "Closure body needs spaces: `{|x| body }` not `{|x|body}`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/style_guide.html#one-line-format")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| {
            let Expr::Closure(block_id) = &expr.expr else {
                return vec![];
            };

            if is_record_type(&expr.ty) || !has_explicit_pipe_delimiters(ctx, expr.span) {
                return vec![];
            }

            check_closure(ctx, expr.span, *block_id)
                .into_iter()
                .collect()
        })
    }

    fn fix(&self, context: &LintContext, &closure_span: &Self::FixInput<'_>) -> Option<Fix> {
        let text = context.span_text(closure_span);
        let pipe_idx = closing_pipe_char_index(text)?;

        Some(Fix::with_explanation(
            "Add spaces around closure body",
            vec![Replacement::new(
                closure_span,
                format_closure(text, pipe_idx),
            )],
        ))
    }
}

pub static RULE: &dyn Rule = &ClosureBodySpacing;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
