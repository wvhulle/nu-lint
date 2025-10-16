use crate::ast_walker::{AstVisitor, VisitContext};
use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use nu_protocol::ast::{Expr, Operator};

#[derive(Default)]
pub struct PreferCompoundAssignment;

impl PreferCompoundAssignment {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for PreferCompoundAssignment {
    fn id(&self) -> &str {
        "S008"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &str {
        "Use compound assignment operators (+=, -=, etc.) for clarity"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut visitor = CompoundAssignmentVisitor::new(self);
        context.walk_ast(&mut visitor);
        visitor.violations
    }
}

/// AST visitor that checks for compound assignment opportunities
struct CompoundAssignmentVisitor<'a> {
    rule: &'a PreferCompoundAssignment,
    violations: Vec<Violation>,
}

impl<'a> CompoundAssignmentVisitor<'a> {
    fn new(rule: &'a PreferCompoundAssignment) -> Self {
        Self {
            rule,
            violations: Vec::new(),
        }
    }
}

impl<'a> AstVisitor for CompoundAssignmentVisitor<'a> {
    fn visit_expression(&mut self, expr: &nu_protocol::ast::Expression, context: &VisitContext) {
        // Look for binary operations that are assignments
        if let Expr::BinaryOp(left, op_expr, right) = &expr.expr
            && let Expr::Operator(nu_protocol::ast::Operator::Assignment(
                nu_protocol::ast::Assignment::Assign,
            )) = &op_expr.expr
        {
            // Found an assignment: var = value
            // Check if the right side is a subexpression containing a binary operation
            if let Expr::Subexpression(block_id) = &right.expr {
                let block = context.working_set.get_block(*block_id);
                // Look for a binary operation in the subexpression
                if let Some(pipeline) = block.pipelines.first()
                    && let Some(element) = pipeline.elements.first()
                    && let Expr::BinaryOp(sub_left, sub_op_expr, _sub_right) = &element.expr.expr
                {
                    if let Expr::Operator(operator) = &sub_op_expr.expr {
                        // Check if left operand matches the variable being assigned to
                        if self.expressions_refer_to_same_variable(left, sub_left, context) {
                            let compound_op = self.get_compound_operator(operator);
                            if let Some(compound_op) = compound_op {
                                let var_text = context.get_span_contents(left.span);
                                let op_symbol = self.get_operator_symbol(operator);

                                // Build fix: extract the right operand from the subexpression
                                let fix = self.build_fix(
                                    var_text,
                                    compound_op,
                                    element,
                                    expr.span,
                                    context,
                                );

                                self.violations.push(Violation {
                                    rule_id: self.rule.id().to_string(),
                                    severity: self.rule.severity(),
                                    message: format!(
                                        "Use compound assignment: {} {} instead of {} = {} {} ...",
                                        var_text, compound_op, var_text, var_text, op_symbol
                                    ),
                                    span: expr.span,
                                    suggestion: Some(format!(
                                        "Replace with: {} {}",
                                        var_text, compound_op
                                    )),
                                    fix,
                                    file: None,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Continue walking the AST
        crate::ast_walker::walk_expression(self, expr, context);
    }
}

impl<'a> CompoundAssignmentVisitor<'a> {
    fn expressions_refer_to_same_variable(
        &self,
        expr1: &nu_protocol::ast::Expression,
        expr2: &nu_protocol::ast::Expression,
        context: &VisitContext,
    ) -> bool {
        // Simple text comparison for now - could be improved with semantic analysis
        let text1 = context.get_span_contents(expr1.span);
        let text2 = context.get_span_contents(expr2.span);
        text1 == text2
    }

    fn build_fix(
        &self,
        var_text: &str,
        compound_op: &str,
        element: &nu_protocol::ast::PipelineElement,
        full_span: nu_protocol::Span,
        context: &VisitContext,
    ) -> Option<crate::context::Fix> {
        use crate::context::{Fix, Replacement};

        // Extract the right operand from the binary operation
        if let Expr::BinaryOp(_left, _op, right) = &element.expr.expr {
            let right_text = context.get_span_contents(right.span);
            let new_text = format!("{var_text} {compound_op} {right_text}");

            Some(Fix {
                description: format!("Replace with compound assignment: {}", new_text),
                replacements: vec![Replacement {
                    span: full_span,
                    new_text,
                }],
            })
        } else {
            None
        }
    }

    fn get_compound_operator(&self, operator: &Operator) -> Option<&'static str> {
        match operator {
            Operator::Math(math_op) => match math_op {
                nu_protocol::ast::Math::Add => Some("+="),
                nu_protocol::ast::Math::Subtract => Some("-="),
                nu_protocol::ast::Math::Multiply => Some("*="),
                nu_protocol::ast::Math::Divide => Some("/="),
                _ => None,
            },
            _ => None,
        }
    }

    fn get_operator_symbol(&self, operator: &Operator) -> &'static str {
        match operator {
            Operator::Math(math_op) => match math_op {
                nu_protocol::ast::Math::Add => "+",
                nu_protocol::ast::Math::Subtract => "-",
                nu_protocol::ast::Math::Multiply => "*",
                nu_protocol::ast::Math::Divide => "/",
                _ => "?",
            },
            _ => "?",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition_assignment_detected() {
        use crate::parser::parse_source;
        use nu_protocol::engine::EngineState;

        let rule = PreferCompoundAssignment::new();

        let bad_code = "$count = $count + 1";
        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, bad_code.as_bytes()).unwrap();
        let context = LintContext {
            source: bad_code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        assert!(
            !rule.check(&context).is_empty(),
            "Should detect $x = $x + 1"
        );
    }

    #[test]
    fn test_subtraction_assignment_detected() {
        use crate::parser::parse_source;
        use nu_protocol::engine::EngineState;

        let rule = PreferCompoundAssignment::new();

        let bad_code = "$value = $value - 5";
        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, bad_code.as_bytes()).unwrap();
        let context = LintContext {
            source: bad_code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        assert!(
            !rule.check(&context).is_empty(),
            "Should detect $x = $x - 5"
        );
    }

    #[test]
    fn test_compound_assignment_not_flagged() {
        use crate::parser::parse_source;
        use nu_protocol::engine::EngineState;

        let rule = PreferCompoundAssignment::new();

        let good_code = "$count += 1";
        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, good_code.as_bytes()).unwrap();
        let context = LintContext {
            source: good_code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag compound assignment"
        );
    }

    #[test]
    fn test_different_variables_not_flagged() {
        use crate::parser::parse_source;
        use nu_protocol::engine::EngineState;

        let rule = PreferCompoundAssignment::new();

        let good_code = "$x = $y + 1";
        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, good_code.as_bytes()).unwrap();
        let context = LintContext {
            source: good_code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag different variables"
        );
    }

    #[test]
    fn test_addition_fix_provided() {
        use crate::parser::parse_source;
        use nu_protocol::engine::EngineState;

        let rule = PreferCompoundAssignment::new();
        let bad_code = "$count = $count + 1";
        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, bad_code.as_bytes()).unwrap();
        let context = LintContext {
            source: bad_code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        let violations = rule.check(&context);
        assert!(!violations.is_empty(), "Should detect issue");
        assert!(violations[0].fix.is_some(), "Should provide a fix");

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements.len(), 1, "Should have one replacement");
        assert_eq!(
            fix.replacements[0].new_text, "$count += 1",
            "Should suggest compound assignment"
        );
    }

    #[test]
    fn test_subtraction_fix_provided() {
        use crate::parser::parse_source;
        use nu_protocol::engine::EngineState;

        let rule = PreferCompoundAssignment::new();
        let bad_code = "$value = $value - 5";
        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, bad_code.as_bytes()).unwrap();
        let context = LintContext {
            source: bad_code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        let violations = rule.check(&context);
        assert!(!violations.is_empty(), "Should detect issue");
        assert!(violations[0].fix.is_some(), "Should provide a fix");

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text, "$value -= 5",
            "Should suggest compound assignment"
        );
    }

    #[test]
    fn test_multiplication_fix_provided() {
        use crate::parser::parse_source;
        use nu_protocol::engine::EngineState;

        let rule = PreferCompoundAssignment::new();
        let bad_code = "$total = $total * 2";
        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, bad_code.as_bytes()).unwrap();
        let context = LintContext {
            source: bad_code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        let violations = rule.check(&context);
        assert!(!violations.is_empty(), "Should detect issue");
        assert!(violations[0].fix.is_some(), "Should provide a fix");

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text, "$total *= 2",
            "Should suggest compound assignment"
        );
    }

    #[test]
    fn test_division_fix_provided() {
        use crate::parser::parse_source;
        use nu_protocol::engine::EngineState;

        let rule = PreferCompoundAssignment::new();
        let bad_code = "$result = $result / 3";
        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, bad_code.as_bytes()).unwrap();
        let context = LintContext {
            source: bad_code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        let violations = rule.check(&context);
        assert!(!violations.is_empty(), "Should detect issue");
        assert!(violations[0].fix.is_some(), "Should provide a fix");

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text, "$result /= 3",
            "Should suggest compound assignment"
        );
    }
}
