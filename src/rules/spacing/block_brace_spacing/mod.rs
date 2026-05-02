use nu_protocol::{Span, ast::Expr};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct BlockBodySpacingFixData {
    block_span: Span,
}

fn check_block_body_spacing(
    context: &LintContext,
    block_span: Span,
) -> Vec<(Detection, BlockBodySpacingFixData)> {
    let text = context.span_text(block_span);

    let mut chars = text.chars();
    if chars.next() != Some('{') || chars.next_back() != Some('}') {
        return vec![];
    }

    let inner: String = chars.collect();
    if inner.trim().is_empty() {
        return vec![];
    }

    let starts_with_space = inner.starts_with(char::is_whitespace);
    let ends_with_space = inner.ends_with(char::is_whitespace);

    if !starts_with_space || !ends_with_space {
        let opening_span = Span::new(block_span.start, block_span.start + 1);
        let closing_span = Span::new(block_span.end - 1, block_span.end);
        vec![(
            Detection::from_global_span(
                "Block body needs spaces inside braces: `{ body }` not `{body}`".to_string(),
                block_span,
            )
            .with_extra_label("add space after `{`", opening_span)
            .with_extra_label("add space before `}`", closing_span),
            BlockBodySpacingFixData { block_span },
        )]
    } else {
        vec![]
    }
}

struct BlockBodySpacing;

impl DetectFix for BlockBodySpacing {
    type FixInput<'a> = BlockBodySpacingFixData;

    fn id(&self) -> &'static str {
        "block_brace_spacing"
    }

    fn short_description(&self) -> &'static str {
        "Block body needs spaces inside braces: `{ body }` not `{body}`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/style_guide.html#one-line-format")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| match &expr.expr {
            // `Type::Any` covers parser-ambiguous cases like `{$name: $value}`
            // which could be either a record or a block.
            Expr::Block(_) if expr.ty != nu_protocol::Type::Any => {
                check_block_body_spacing(ctx, expr.span)
            }
            _ => vec![],
        })
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let text = context.span_text(fix_data.block_span);

        let mut chars = text.chars();
        if chars.next() != Some('{') || chars.next_back() != Some('}') {
            return None;
        }
        let inner: String = chars.collect();
        let trimmed = inner.trim();
        let fixed = format!("{{ {trimmed} }}");

        Some(Fix {
            explanation: "Add spaces inside block braces".into(),
            replacements: vec![Replacement::new(fix_data.block_span, fixed)],
        })
    }
}

pub static RULE: &dyn Rule = &BlockBodySpacing;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
