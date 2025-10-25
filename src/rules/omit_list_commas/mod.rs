use nu_protocol::{Span, ast::Expr};

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
};

fn check_list_commas(
    source: &str,
    span: Span,
    items: &[nu_protocol::ast::ListItem],
) -> Vec<Violation> {
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
            nu_protocol::ast::ListItem::Item(expr)
            | nu_protocol::ast::ListItem::Spread(_, expr) => expr,
        };
        let next_expr = match &items[i + 1] {
            nu_protocol::ast::ListItem::Item(expr)
            | nu_protocol::ast::ListItem::Spread(_, expr) => expr,
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

                violations.push(Violation {
                    rule_id: "omit_list_commas".into(),
                    severity: Severity::Info,
                    message: "Omit commas between list items".to_string().into(),
                    span: comma_span,
                    suggestion: Some(
                        "Remove the comma - Nushell lists don't need commas"
                            .to_string()
                            .into(),
                    ),
                    fix: None,
                    file: None,
                });
            }
        }
    }

    violations
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_violations(|expr, ctx| match &expr.expr {
        Expr::List(items) => check_list_commas(ctx.source, expr.span, items),
        _ => vec![],
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "omit_list_commas",
        RuleCategory::Formatting,
        Severity::Info,
        "Omit commas between list items as per Nushell style guide",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
