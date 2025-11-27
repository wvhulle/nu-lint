use nu_protocol::{
    Span,
    ast::{Expr, ListItem},
};

use crate::{context::LintContext, rule::Rule, violation::Violation};
fn check_list_commas(source: &str, span: Span, items: &[ListItem]) -> Vec<Violation> {
    let mut violations = Vec::new();
    if items.len() < 2 || span.end > source.len() {
        return violations;
    }
    let list_text = &source[span.start..span.end];
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
        let between_start = current_expr.span.end;
        let between_end = next_expr.span.start;
        if between_start >= between_end || between_end > source.len() {
            continue;
        }
        let between_text = &source[between_start..between_end];
        if between_text.contains(',') {
            // Find the comma position for precise span
            if let Some(comma_pos) = between_text.find(',') {
                let comma_span =
                    Span::new(between_start + comma_pos, between_start + comma_pos + 1);
                violations.push(
                    Violation::new("Omit commas between list items", comma_span)
                        .with_help("Remove the comma - Nushell lists don't need commas"),
                );
            }
        }
    }
    violations
}
fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| match &expr.expr {
        Expr::List(items) => check_list_commas(ctx.source, expr.span, items),
        _ => vec![],
    })
}
pub const fn rule() -> Rule {
    Rule::new(
        "omit_list_commas",
        "Omit commas between list items as per Nushell style guide",
        check,
    )
    .with_doc_url("https://www.nushell.sh/book/style_guide.html#basic")
}
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
