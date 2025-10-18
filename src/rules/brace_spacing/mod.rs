use nu_protocol::{Span, ast::Expr};

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{AstRule, RuleCategory, RuleMetadata},
    visitor::{AstVisitor, VisitContext},
};

#[derive(Default)]
pub struct BraceSpacing;

impl RuleMetadata for BraceSpacing {
    fn id(&self) -> &'static str {
        "brace_spacing"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Formatting
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &'static str {
        "Braces should have consistent spacing: either {x} or { x }, and no space before closure \
         parameters"
    }
}

impl AstRule for BraceSpacing {
    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut visitor = BraceSpacingVisitor::new(self, context.source);
        context.walk_ast(&mut visitor);
        visitor.violations
    }
}

/// AST visitor that checks for brace spacing issues
pub struct BraceSpacingVisitor<'a> {
    rule: &'a BraceSpacing,
    source: &'a str,
    violations: Vec<Violation>,
}

impl<'a> BraceSpacingVisitor<'a> {
    #[must_use]
    pub fn new(rule: &'a BraceSpacing, source: &'a str) -> Self {
        Self {
            rule,
            source,
            violations: Vec::new(),
        }
    }

    fn check_brace_spacing(&mut self, span: Span, has_params: bool) {
        if span.start >= span.end || span.end > self.source.len() {
            return;
        }

        let text = &self.source[span.start..span.end];

        // Find opening and closing braces
        if !text.starts_with('{') || !text.ends_with('}') {
            return;
        }

        let inner = &text[1..text.len() - 1];

        // Empty braces are fine
        if inner.trim().is_empty() {
            return;
        }

        // Check for space after opening brace before closure parameters
        if has_params && inner.starts_with(|c: char| c.is_whitespace()) {
            let pipe_pos = inner.find('|');
            if let Some(pos) = pipe_pos
                && pos > 0
                && inner[..pos].trim().is_empty()
            {
                self.violations.push(Violation {
                    rule_id: self.rule.id().to_string(),
                    severity: self.rule.severity(),
                    message: "No space allowed after opening brace before closure parameters"
                        .to_string(),
                    span,
                    suggestion: Some("Use {|param| instead of { |param|".to_string()),
                    fix: None,
                    file: None,
                });
                return;
            }
        }

        // Skip closure parameter checking for other cases
        if has_params {
            return;
        }

        // Check for inconsistent spacing in records/blocks
        let starts_with_space = inner.starts_with(|c: char| c.is_whitespace());
        let ends_with_space = inner.ends_with(|c: char| c.is_whitespace());

        // Inconsistent: one has space, the other doesn't
        if starts_with_space != ends_with_space {
            self.violations.push(Violation {
                rule_id: self.rule.id().to_string(),
                severity: self.rule.severity(),
                message: "Inconsistent brace spacing: use either {x} or { x }, not { x} or {x }"
                    .to_string(),
                span,
                suggestion: Some(
                    "Use consistent spacing: both spaces or no spaces inside braces".to_string(),
                ),
                fix: None,
                file: None,
            });
        }
    }
}

impl AstVisitor for BraceSpacingVisitor<'_> {
    fn visit_expression(&mut self, expr: &nu_protocol::ast::Expression, context: &VisitContext) {
        match &expr.expr {
            // Closures and blocks with parameters
            Expr::Closure(block_id) | Expr::Block(block_id) => {
                let block = context.get_block(*block_id);
                let has_params = !block.signature.required_positional.is_empty()
                    || !block.signature.optional_positional.is_empty()
                    || block.signature.rest_positional.is_some();

                self.check_brace_spacing(expr.span, has_params);
            }
            // Records
            Expr::Record(items) => {
                if !items.is_empty() {
                    self.check_brace_spacing(expr.span, false);
                }
            }
            _ => {}
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
