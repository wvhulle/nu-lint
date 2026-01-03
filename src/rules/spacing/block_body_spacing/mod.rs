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
    let text = context.plain_text(block_span);

    // Validate basic structure using char iterators for UTF-8 safety
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
                "Blocks and closures without parameters should have spaces inside curly braces"
                    .to_string(),
                block_span,
            )
            .with_extra_label("needs space after", opening_span)
            .with_extra_label("needs space before", closing_span)
            .with_help("Use { body } for blocks without parameters"),
            BlockBodySpacingFixData { block_span },
        )]
    } else {
        vec![]
    }
}

fn has_block_params(context: &LintContext, block_id: nu_protocol::BlockId) -> bool {
    let block = context.working_set.get_block(block_id);
    !block.signature.required_positional.is_empty()
        || !block.signature.optional_positional.is_empty()
        || block.signature.rest_positional.is_some()
}

const fn is_record_type(ty: &nu_protocol::Type) -> bool {
    matches!(ty, nu_protocol::Type::Record(_))
}

struct BlockBodySpacing;

impl DetectFix for BlockBodySpacing {
    type FixInput<'a> = BlockBodySpacingFixData;

    fn id(&self) -> &'static str {
        "curly_block_body_spacing"
    }

    fn explanation(&self) -> &'static str {
        "Blocks and closures without parameters should have spaces inside curly braces"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/style_guide.html#one-line-format")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| {
            match &expr.expr {
                Expr::Closure(block_id) | Expr::Block(block_id) => {
                    // Skip records (they have different spacing rules)
                    if is_record_type(&expr.ty) {
                        return vec![];
                    }
                    // Only check blocks/closures without parameters
                    if !has_block_params(ctx, *block_id) {
                        return check_block_body_spacing(ctx, expr.span);
                    }
                    vec![]
                }
                _ => vec![],
            }
        })
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let text = context.plain_text(fix_data.block_span);

        // Extract inner content using char iterators for UTF-8 safety
        let mut chars = text.chars();
        if chars.next() != Some('{') || chars.next_back() != Some('}') {
            return None;
        }
        let inner: String = chars.collect();
        let trimmed = inner.trim();
        let fixed = format!("{{ {trimmed} }}");

        Some(Fix::with_explanation(
            "Add spaces inside block braces",
            vec![Replacement::new(fix_data.block_span, fixed)],
        ))
    }
}

pub static RULE: &dyn Rule = &BlockBodySpacing;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
