use nu_protocol::{
    Span,
    ast::{Expr, ListItem},
};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn check_list_commas(context: &LintContext, span: Span, items: &[ListItem]) -> Vec<Detection> {
    let mut violations = Vec::new();
    if items.len() < 2 {
        return violations;
    }
    let list_text = context.get_span_text(span);
    // Skip if not a bracket list
    if !list_text.trim_start().starts_with('[') {
        return violations;
    }
    // Check for commas between list items
    for i in 0..items.len() - 1 {
        let current_expr = match &items[i] {
            ListItem::Item(expr) | ListItem::Spread(_, expr) => expr,
        };
        let next_expr = match &items[i + 1] {
            ListItem::Item(expr) | ListItem::Spread(_, expr) => expr,
        };
        if current_expr.span.end >= next_expr.span.start {
            continue;
        }
        let between_span = Span::new(current_expr.span.end, next_expr.span.start);
        let between_text = context.get_span_text(between_span);
        if between_text.contains(',') {
            // Find the comma position for precise span (use global span - will be
            // normalized later)
            if let Some(comma_pos) = between_text.find(',') {
                let comma_span = Span::new(
                    between_span.start + comma_pos,
                    between_span.start + comma_pos + 1,
                );
                violations.push(
                    Detection::from_global_span("Omit commas between list items", comma_span)
                        .with_help("Remove the comma - Nushell lists don't need commas"),
                );
            }
        }
    }
    violations
}

struct OmitListCommas;

impl DetectFix for OmitListCommas {
    type FixInput = ();

    fn id(&self) -> &'static str {
        "omit_list_commas"
    }

    fn explanation(&self) -> &'static str {
        "Omit commas between list items."
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/style_guide.html#basic")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
        let violations = context.detect(|expr, ctx| match &expr.expr {
            Expr::List(items) => check_list_commas(ctx, expr.span, items),
            _ => vec![],
        });
        Self::no_fix(violations)
    }
}

pub static RULE: &dyn Rule = &OmitListCommas;
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
