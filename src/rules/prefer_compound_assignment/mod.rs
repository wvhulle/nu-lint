use nu_protocol::ast::{Expr, Operator};

use crate::{
    context::LintContext,
    lint::{Fix, Replacement, Severity, Violation},
    rule::{Rule, RuleCategory},
    visitor::{AstVisitor, VisitContext},
};

/// AST visitor that checks for compound assignment opportunities
struct CompoundAssignmentVisitor {
    violations: Vec<Violation>,
}

impl CompoundAssignmentVisitor {
    fn new() -> Self {
        Self {
            violations: Vec::new(),
        }
    }

    fn expressions_refer_to_same_variable(
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
        var_text: &str,
        compound_op: &str,
        element: &nu_protocol::ast::PipelineElement,
        full_span: nu_protocol::Span,
        context: &VisitContext,
    ) -> Option<Fix> {
        // Extract the right operand from the binary operation
        if let Expr::BinaryOp(_left, _op, right) = &element.expr.expr {
            let right_text = context.get_span_contents(right.span);
            let new_text = format!("{var_text} {compound_op} {right_text}");

            Some(Fix {
                description: format!("Replace with compound assignment: {new_text}").into(),
                replacements: vec![Replacement {
                    span: full_span,
                    new_text: new_text.into(),
                }],
            })
        } else {
            None
        }
    }

    fn get_compound_operator(operator: Operator) -> Option<&'static str> {
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

    fn get_operator_symbol(operator: Operator) -> &'static str {
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

impl AstVisitor for CompoundAssignmentVisitor {
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
                    && let Expr::Operator(operator) = &sub_op_expr.expr
                {
                    // Check if left operand matches the variable being assigned to
                    if Self::expressions_refer_to_same_variable(left, sub_left, context) {
                        let compound_op = Self::get_compound_operator(*operator);
                        if let Some(compound_op) = compound_op {
                            let var_text = context.get_span_contents(left.span);
                            let op_symbol = Self::get_operator_symbol(*operator);

                            // Build fix: extract the right operand from the subexpression
                            let fix =
                                Self::build_fix(var_text, compound_op, element, expr.span, context);

                            self.violations.push(Violation {
                                rule_id: "prefer_compound_assignment".into(),
                                severity: Severity::Info,
                                message: format!(
                                    "Use compound assignment: {var_text} {compound_op} instead of \
                                     {var_text} = {var_text} {op_symbol} ..."
                                )
                                .into(),
                                span: expr.span,
                                suggestion: Some(
                                    format!("Replace with: {var_text} {compound_op}").into(),
                                ),
                                fix,
                                file: None,
                            });
                        }
                    }
                }
            }
        }

        crate::visitor::walk_expression(self, expr, context);
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    let mut visitor = CompoundAssignmentVisitor::new();
    context.walk_ast(&mut visitor);
    visitor.violations
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_compound_assignment",
        RuleCategory::Idioms,
        Severity::Info,
        "Use compound assignment operators (+=, -=, etc.) for clarity",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
