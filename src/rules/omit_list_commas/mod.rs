use nu_protocol::{Span, ast::Expr};

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
    visitor::{AstVisitor, VisitContext},
};

/// AST visitor that checks for unnecessary commas in lists
struct OmitListCommasVisitor<'a> {
    source: &'a str,
    violations: Vec<Violation>,
}

impl<'a> OmitListCommasVisitor<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            violations: Vec::new(),
        }
    }

    fn check_list_commas(&mut self, span: Span, items: &[nu_protocol::ast::ListItem]) {
        if items.len() < 2 || span.end > self.source.len() {
            return;
        }

        let list_text = &self.source[span.start..span.end];

        // Skip if not a bracket list
        if !list_text.trim_start().starts_with('[') {
            return;
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

            if between_start >= between_end || between_end > self.source.len() {
                continue;
            }

            let between_text = &self.source[between_start..between_end];

            if between_text.contains(',') {
                // Find the comma position for precise span
                if let Some(comma_pos) = between_text.find(',') {
                    let comma_span =
                        Span::new(between_start + comma_pos, between_start + comma_pos + 1);

                    self.violations.push(Violation {
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
    }
}

impl AstVisitor for OmitListCommasVisitor<'_> {
    fn visit_expression(&mut self, expr: &nu_protocol::ast::Expression, context: &VisitContext) {
        if let Expr::List(items) = &expr.expr {
            self.check_list_commas(expr.span, items);
        }

        // Continue walking the tree
        crate::visitor::walk_expression(self, expr, context);
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    let mut visitor = OmitListCommasVisitor::new(context.source);
    context.walk_ast(&mut visitor);
    visitor.violations
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
