use nu_protocol::{Span, ast::Expr};

use super::has_explicit_pipe_delimiters;
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
    let text = context.span_text(closure_span);

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
            "Closure opening brace should directly touch parameter pipe `{|`".to_string(),
            opening_brace_span,
        )
        .with_primary_label("`{` here")
        .with_extra_label("remove this whitespace", whitespace_span)
        .with_extra_label("`|` should follow `{` directly", pipe_span),
        ClosureParamSpacingFixData { whitespace_span },
    )]
}

struct ClosureParamSpacing;

impl DetectFix for ClosureParamSpacing {
    type FixInput<'a> = ClosureParamSpacingFixData;

    fn id(&self) -> &'static str {
        "closure_brace_pipe_spacing"
    }

    fn short_description(&self) -> &'static str {
        "Space between `{` and `|` in closure"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/style_guide.html#one-line-format")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| match &expr.expr {
            // Check any closure with explicit pipe delimiters (including empty `||`)
            Expr::Closure(_) if has_explicit_pipe_delimiters(ctx, expr.span) => {
                check_closure_param_spacing(ctx, expr.span)
            }
            _ => vec![],
        })
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        // Simply remove the whitespace between the opening brace and the pipe
        Some(Fix {
            explanation: "Remove space before closure parameters".into(),
            replacements: vec![Replacement::new(fix_data.whitespace_span, String::new())],
        })
    }
}

pub static RULE: &dyn Rule = &ClosureParamSpacing;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
