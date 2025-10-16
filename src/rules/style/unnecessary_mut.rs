use crate::context::LintContext;
use crate::lint::{Severity, Violation};
use crate::rule::{Rule, RuleCategory};
use crate::visitor::{AstVisitor, VisitContext};
use nu_protocol::ast::Expr;
use nu_protocol::{Span, VarId};
use std::collections::HashMap;

pub struct UnnecessaryMut;

impl UnnecessaryMut {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UnnecessaryMut {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for UnnecessaryMut {
    fn id(&self) -> &str {
        "S015"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &str {
        "Variables should only be marked 'mut' when they are actually reassigned"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut visitor = MutVariableVisitor::new(self, context.source);
        context.walk_ast(&mut visitor);
        visitor.finalize()
    }
}

/// AST visitor that tracks mutable variable declarations and assignments
struct MutVariableVisitor<'a> {
    rule: &'a UnnecessaryMut,
    /// Maps variable IDs to their (name, declaration_span, mut_keyword_span, is_reassigned)
    mut_variables: HashMap<VarId, (String, Span, Span, bool)>,
    source: &'a str,
}

impl<'a> MutVariableVisitor<'a> {
    fn new(rule: &'a UnnecessaryMut, source: &'a str) -> Self {
        Self {
            rule,
            mut_variables: HashMap::new(),
            source,
        }
    }

    /// Find the span of 'mut ' keyword before the variable name
    /// Looks backwards from the variable name span to find 'mut '
    fn find_mut_keyword_span(&self, var_span: Span) -> Span {
        // Look backwards in the source before the variable name
        // Pattern is: "mut variable_name", so we look for "mut " before the var name
        let start = var_span.start;

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
        use crate::lint::{Fix, Replacement};
        let mut violations = Vec::new();

        // Check which mutable variables were never reassigned
        for (var_name, decl_span, mut_span, is_reassigned) in self.mut_variables.values() {
            if !is_reassigned {
                // Create a fix that removes the 'mut ' keyword
                let fix = Some(Fix {
                    description: format!("Remove 'mut' keyword from variable '{}'", var_name),
                    replacements: vec![Replacement {
                        span: *mut_span,
                        new_text: String::new(), // Replace 'mut ' with empty string
                    }],
                });

                violations.push(Violation {
                    rule_id: self.rule.id().to_string(),
                    severity: self.rule.severity(),
                    message: format!(
                        "Variable '{}' is declared as 'mut' but never reassigned",
                        var_name
                    ),
                    span: *decl_span,
                    suggestion: Some(format!("Remove 'mut' keyword:\nlet {} = ...", var_name)),
                    fix,
                    file: None,
                });
            }
        }

        violations
    }
}

impl<'a> AstVisitor for MutVariableVisitor<'a> {
    fn visit_var_decl(&mut self, var_id: VarId, span: Span, context: &VisitContext) {
        let var = context.get_variable(var_id);
        if var.mutable {
            // The span only covers the variable name itself (e.g., just "x")
            // Get the variable name directly from the span
            let var_name = context.get_span_contents(span).to_string();

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
        // Check if this is any assignment operation (including compound assignments like +=, -=, etc.)
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
                    if let Expr::Var(var_id) = &cell_path.head.expr {
                        if let Some((_, _, _, is_reassigned)) = self.mut_variables.get_mut(var_id) {
                            *is_reassigned = true;
                        }
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

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::engine::LintEngine;

    #[test]
    fn test_unnecessary_mut_detected() {
        let source = r"
def process [] {
    mut x = 5
    echo $x
}
";
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None);

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "S015").collect();

        assert!(!rule_violations.is_empty(), "Should detect unnecessary mut");
        assert!(rule_violations[0].message.contains("never reassigned"));
    }

    #[test]
    fn test_necessary_mut_not_flagged() {
        let source = r"
def fibonacci [n: int] {
    mut a = 0
    mut b = 1
    for _ in 2..=$n {
        let c = $a + $b
        $a = $b
        $b = $c
    }
    $b
}
";
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None);

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "S015").collect();

        assert!(
            rule_violations.is_empty(),
            "Should not flag mut variables that are reassigned"
        );
    }

    #[test]
    fn test_immutable_variable_not_flagged() {
        let source = r"
def process [] {
    let x = 5
    echo $x
}
";
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None);

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "S015").collect();

        assert!(
            rule_violations.is_empty(),
            "Should not flag immutable variables"
        );
    }

    #[test]
    fn test_mut_with_compound_assignment() {
        let source = r"
def increment [] {
    mut counter = 0
    $counter += 1
    echo $counter
}
";
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None);

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "S015").collect();

        assert!(
            rule_violations.is_empty(),
            "Should not flag mut with compound assignment"
        );
    }

    #[test]
    fn test_underscore_prefixed_mut_not_flagged() {
        let source = r#"
def process [] {
    mut _temp = 5
    echo "done"
}
"#;
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None);

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "S015").collect();

        assert!(
            rule_violations.is_empty(),
            "Should not flag underscore-prefixed mut variables"
        );
    }

    #[test]
    fn test_multiple_mut_variables() {
        let source = r"
def process [] {
    mut a = 1
    mut b = 2
    mut c = 3
    $a = 10
    $c = 30
    echo $a $b $c
}
";
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None);

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "S015").collect();

        // Only 'b' should be flagged as unnecessary mut
        assert_eq!(
            rule_violations.len(),
            1,
            "Should flag only the one unnecessary mut"
        );
        assert!(
            rule_violations[0].message.contains("b"),
            "Should flag variable 'b'"
        );
    }

    #[test]
    fn test_unnecessary_mut_fix_provided() {
        let source = r"
def process [] {
    mut x = 5
    echo $x
}
";
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None);

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "S015").collect();

        assert!(!rule_violations.is_empty(), "Should detect unnecessary mut");
        assert!(rule_violations[0].fix.is_some(), "Should provide a fix");

        let fix = rule_violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements.len(), 1, "Should have one replacement");
        // The fix should remove 'mut ' from the declaration
        assert!(
            fix.description.contains("Remove 'mut'"),
            "Fix description should mention removing 'mut'"
        );
    }

    #[test]
    fn test_necessary_mut_no_fix() {
        let source = r#"
def increment [] {
    mut counter = 0
    $counter += 1
    echo $counter
}
"#;
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None);

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "S015").collect();

        assert!(
            rule_violations.is_empty(),
            "Should not flag necessary mut variables"
        );
    }
}
