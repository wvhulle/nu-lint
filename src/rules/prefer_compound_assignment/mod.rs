use nu_protocol::ast::{Expr, Operator};

use crate::{
    ast_utils::ExpressionExt,
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{Fix, Replacement, RuleViolation, Severity},
};

fn build_fix(
    var_text: &str,
    compound_op: &str,
    element: &nu_protocol::ast::PipelineElement,
    full_span: nu_protocol::Span,
    context: &LintContext,
) -> Option<Fix> {
    // Extract the right operand from the binary operation
    if let Expr::BinaryOp(_left, _op, right) = &element.expr.expr {
        let right_text = right.span_text(context);
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

fn check_for_compound_assignment(
    expr: &nu_protocol::ast::Expression,
    ctx: &LintContext,
) -> Option<RuleViolation> {
    let Expr::BinaryOp(left, op_expr, right) = &expr.expr else {
        return None;
    };

    let Expr::Operator(nu_protocol::ast::Operator::Assignment(
        nu_protocol::ast::Assignment::Assign,
    )) = &op_expr.expr
    else {
        return None;
    };

    let Expr::Subexpression(block_id) = &right.expr else {
        return None;
    };

    let block = ctx.working_set.get_block(*block_id);

    let pipeline = block.pipelines.first()?;
    let element = pipeline.elements.first()?;

    let Expr::BinaryOp(sub_left, sub_op_expr, _sub_right) = &element.expr.expr else {
        return None;
    };

    let Expr::Operator(operator) = &sub_op_expr.expr else {
        return None;
    };

    if !left.refers_to_same_variable(sub_left, ctx) {
        return None;
    }

    let compound_op = get_compound_operator(*operator)?;

    let var_text = left.span_text(ctx);
    let op_symbol = get_operator_symbol(*operator);

    let fix = build_fix(var_text, compound_op, element, expr.span, ctx);

    let violation = RuleViolation::new_dynamic(
        "prefer_compound_assignment",
        format!(
            "Use compound assignment: {var_text} {compound_op} instead of {var_text} = {var_text} \
             {op_symbol} ..."
        ),
        expr.span,
    )
    .with_suggestion_dynamic(format!("Replace with: {var_text} {compound_op}"));

    let violation = match fix {
        Some(f) => violation.with_fix(f),
        None => violation,
    };

    Some(violation)
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context.collect_rule_violations(|expr, ctx| {
        check_for_compound_assignment(expr, ctx)
            .into_iter()
            .collect()
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
