use nu_protocol::{Span, ast::Expr};

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{AstRule, RuleCategory, RuleMetadata},
    visitor::{AstVisitor, VisitContext},
};

#[derive(Default)]
pub struct OmitListCommas;

impl RuleMetadata for OmitListCommas {
    fn id(&self) -> &'static str {
        "omit_list_commas"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Formatting
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &'static str {
        "Omit commas between list items as per Nushell style guide"
    }
}

impl AstRule for OmitListCommas {
    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut visitor = OmitListCommasVisitor::new(self, context.source);
        context.walk_ast(&mut visitor);
        visitor.violations
    }
}

/// AST visitor that checks for unnecessary commas in lists
pub struct OmitListCommasVisitor<'a> {
    rule: &'a OmitListCommas,
    source: &'a str,
    violations: Vec<Violation>,
}

impl<'a> OmitListCommasVisitor<'a> {
    #[must_use]
    pub fn new(rule: &'a OmitListCommas, source: &'a str) -> Self {
        Self {
            rule,
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
                nu_protocol::ast::ListItem::Item(expr) | nu_protocol::ast::ListItem::Spread(_, expr) => expr,
            };
            let next_expr = match &items[i + 1] {
                nu_protocol::ast::ListItem::Item(expr) | nu_protocol::ast::ListItem::Spread(_, expr) => expr,
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
                    let comma_span = Span::new(
                        between_start + comma_pos,
                        between_start + comma_pos + 1,
                    );

                    self.violations.push(Violation {
                        rule_id: self.rule.id().to_string(),
                        severity: self.rule.severity(),
                        message: "Omit commas between list items".to_string(),
                        span: comma_span,
                        suggestion: Some("Remove the comma - Nushell lists don't need commas".to_string()),
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

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;