use std::collections::HashMap;

use nu_protocol::{Span, VarId, ast::Expr};

use crate::{
    context::LintContext,
    lint::{Fix, Replacement, Severity, Violation},
    rule::{Rule, RuleCategory},
    visitor::{AstVisitor, VisitContext},
};

/// AST visitor that tracks mutable variable declarations and assignments
struct MutVariableVisitor<'a> {
    /// Maps variable IDs to their (name, `declaration_span`,
    /// `mut_keyword_span`, `is_reassigned`)
    mut_variables: HashMap<VarId, (String, Span, Span, bool)>,
    source: &'a str,
}

impl<'a> MutVariableVisitor<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            mut_variables: HashMap::new(),
            source,
        }
    }

    /// Find the span of 'mut ' keyword before the variable name
    /// Looks backwards from the variable name span to find 'mut '
    fn find_mut_keyword_span(&self, var_span: Span) -> Span {
        // Look backwards in the source before the variable name
        // Pattern is: "mut variable_name", so we look for "mut " before the var name
        let start = var_span.start.min(self.source.len());

        // Search backwards for "mut " (including the space)
        // We limit the search to a reasonable distance (e.g., 20 chars back)
        let search_start = start.saturating_sub(20);
        let text_before = &self.source[search_start..start];

        if let Some(mut_pos) = text_before.rfind("mut ") {
            let abs_mut_start = search_start + mut_pos;
            let abs_mut_end = abs_mut_start + 4; // "mut " is 4 characters
            return Span::new(abs_mut_start, abs_mut_end);
        }

        // Fallback: if we can't find it, return a span covering the variable
        // This shouldn't happen in well-formed code
        var_span
    }

    /// Generate violations after AST traversal is complete
    fn finalize(self) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Check which mutable variables were never reassigned
        for (var_name, decl_span, mut_span, is_reassigned) in self.mut_variables.values() {
            if !is_reassigned {
                // Create a fix that removes the 'mut ' keyword
                let fix = Some(Fix {
                    description: format!("Remove 'mut' keyword from variable '{var_name}'").into(),
                    replacements: vec![Replacement {
                        span: *mut_span,
                        new_text: String::new().into(), // Replace 'mut ' with empty string
                    }],
                });

                violations.push(Violation {
                    rule_id: "unnecessary_mut".into(),
                    severity: Severity::Info,
                    message: format!(
                        "Variable '{var_name}' is declared as 'mut' but never reassigned"
                    ).into(),
                    span: *decl_span,
                    suggestion: Some(format!("Remove 'mut' keyword:\nlet {var_name} = ...").into()),
                    fix,
                    file: None,
                });
            }
        }

        violations
    }
}

impl AstVisitor for MutVariableVisitor<'_> {
    fn visit_var_decl(&mut self, var_id: VarId, span: Span, context: &VisitContext) {
        let var = context.get_variable(var_id);
        if var.mutable {
            // The span only covers the variable name itself (e.g., just "x")
            // Get the variable name directly from the span
            let var_name: String = context.get_span_contents(span).to_string();

            // Skip underscore-prefixed variables (convention for intentionally unused)
            if !var_name.starts_with('_') {
                // Find the 'mut' keyword before the variable name
                // We need to look back in the source to find "mut "
                let mut_span = self.find_mut_keyword_span(span);
                self.mut_variables
                    .insert(var_id, (var_name, span, mut_span, false));
            }
        }
    }

    fn visit_binary_op(
        &mut self,
        lhs: &nu_protocol::ast::Expression,
        op: &nu_protocol::ast::Expression,
        rhs: &nu_protocol::ast::Expression,
        context: &VisitContext,
    ) {
        // Check if this is any assignment operation (including compound assignments
        // like +=, -=, etc.)
        if let Expr::Operator(nu_protocol::ast::Operator::Assignment(_)) = &op.expr {
            // Mark the left-hand side variable as reassigned
            // In Nushell, variables in assignments are FullCellPath expressions,
            // not plain Var expressions
            match &lhs.expr {
                Expr::Var(var_id) => {
                    // Direct variable (less common)
                    if let Some((_, _, _, is_reassigned)) = self.mut_variables.get_mut(var_id) {
                        *is_reassigned = true;
                    }
                }
                Expr::FullCellPath(cell_path) => {
                    // Variable with optional cell path (e.g., $x or $x.field)
                    if let Expr::Var(var_id) = &cell_path.head.expr
                        && let Some((_, _, _, is_reassigned)) = self.mut_variables.get_mut(var_id)
                    {
                        *is_reassigned = true;
                    }
                }
                _ => {}
            }
        }

        // Continue walking the child expressions using the default implementation
        self.visit_expression(lhs, context);
        self.visit_expression(op, context);
        self.visit_expression(rhs, context);
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    let mut visitor = MutVariableVisitor::new(context.source);
    context.walk_ast(&mut visitor);
    visitor.finalize()
}

pub fn rule() -> Rule {
    Rule::new(
        "unnecessary_mut",
        RuleCategory::CodeQuality,
        Severity::Info,
        "Variables should only be marked 'mut' when they are actually reassigned",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
