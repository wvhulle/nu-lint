use nu_protocol::ast::{Expr, Operator};

use crate::{
    context::LintContext,
    lint::{Fix, Replacement, RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn expressions_refer_to_same_variable(
    expr1: &nu_protocol::ast::Expression,
    expr2: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> bool {
    // Simple text comparison for now - could be improved with semantic analysis
    let text1 = &context.source[expr1.span.start..expr1.span.end];
    let text2 = &context.source[expr2.span.start..expr2.span.end];
    text1 == text2
}

fn build_fix(
    var_text: &str,
    compound_op: &str,
    element: &nu_protocol::ast::PipelineElement,
    full_span: nu_protocol::Span,
    context: &LintContext,
) -> Option<Fix> {
    // Extract the right operand from the binary operation
    if let Expr::BinaryOp(_left, _op, right) = &element.expr.expr {
        let right_text = &context.source[right.span.start..right.span.end];
        let new_text = format!("{var_text} {compound_op} {right_text}");

        Some(Fix::new_dynamic(
            format!("Replace with compound assignment: {new_text}"),
            vec![Replacement::new_dynamic(full_span, new_text)],
        ))
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

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context.collect_rule_violations(|expr, ctx| {
        // Look for binary operations that are assignments
        if let Expr::BinaryOp(left, op_expr, right) = &expr.expr
            && let Expr::Operator(nu_protocol::ast::Operator::Assignment(
                nu_protocol::ast::Assignment::Assign,
            )) = &op_expr.expr
        {
            // Found an assignment: var = value
            // Check if the right side is a subexpression containing a binary operation
            if let Expr::Subexpression(block_id) = &right.expr {
                let block = ctx.working_set.get_block(*block_id);
                // Look for a binary operation in the subexpression
                if let Some(pipeline) = block.pipelines.first()
                    && let Some(element) = pipeline.elements.first()
                    && let Expr::BinaryOp(sub_left, sub_op_expr, _sub_right) = &element.expr.expr
                    && let Expr::Operator(operator) = &sub_op_expr.expr
                {
                    // Check if left operand matches the variable being assigned to
                    if expressions_refer_to_same_variable(left, sub_left, ctx) {
                        let compound_op = get_compound_operator(*operator);
                        if let Some(compound_op) = compound_op {
                            let var_text = &ctx.source[left.span.start..left.span.end];
                            let op_symbol = get_operator_symbol(*operator);

                            // Build fix: extract the right operand from the subexpression
                            let fix = build_fix(var_text, compound_op, element, expr.span, ctx);

                            let mut violation = RuleViolation::new_dynamic(
                                "prefer_compound_assignment",
                                format!(
                                    "Use compound assignment: {var_text} {compound_op} instead of \
                                     {var_text} = {var_text} {op_symbol} ..."
                                ),
                                expr.span,
                            )
                            .with_suggestion_dynamic(format!(
                                "Replace with: {var_text} {compound_op}"
                            ));

                            if let Some(fix) = fix {
                                violation = violation.with_fix(fix);
                            }

                            return vec![violation];
                        }
                    }
                }
            }
        }
        vec![]
    })
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
