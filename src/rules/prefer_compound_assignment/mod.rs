use nu_protocol::ast::{Assignment, Expr, Expression, Math, Operator, PipelineElement};

use crate::{
    LintLevel,
    ast::expression::ExpressionExt,
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement, Violation},
};

fn build_fix(
    var_text: &str,
    compound_op: &str,
    element: &PipelineElement,
    full_span: nu_protocol::Span,
    context: &LintContext,
) -> Option<Fix> {
    // Extract the right operand from the binary operation
    if let Expr::BinaryOp(_left, _op, right) = &element.expr.expr {
        let right_text = right.span_text(context);
        let new_text = format!("{var_text} {compound_op} {right_text}");

        Some(Fix::with_explanation(
            format!("Replace with compound assignment: {new_text}"),
            vec![Replacement::new(full_span, new_text)],
        ))
    } else {
        None
    }
}

const fn get_compound_operator(operator: Operator) -> Option<&'static str> {
    match operator {
        Operator::Math(math_op) => match math_op {
            Math::Add => Some("+="),
            Math::Subtract => Some("-="),
            Math::Multiply => Some("*="),
            Math::Divide => Some("/="),
            _ => None,
        },
        _ => None,
    }
}

const fn get_operator_symbol(operator: Operator) -> &'static str {
    match operator {
        Operator::Math(math_op) => match math_op {
            Math::Add => "+",
            Math::Subtract => "-",
            Math::Multiply => "*",
            Math::Divide => "/",
            _ => "?",
        },
        _ => "?",
    }
}

fn check_for_compound_assignment(expr: &Expression, ctx: &LintContext) -> Option<Violation> {
    let Expr::BinaryOp(left, op_expr, right) = &expr.expr else {
        return None;
    };

    let Expr::Operator(Operator::Assignment(Assignment::Assign)) = &op_expr.expr else {
        return None;
    };

    let Expr::Subexpression(block_id) = &right.expr else {
        return None;
    };

    let block = ctx.working_set.get_block(*block_id);

    let pipeline = block.pipelines.first()?;
    let element = pipeline.elements.first()?;

    let Expr::BinaryOp(sub_left, sub_op_expr, sub_right) = &element.expr.expr else {
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

    let violation = Violation::new(
        format!(
            "Use compound assignment: {var_text} {compound_op} instead of {var_text} = {var_text} \
             {op_symbol} ..."
        ),
        left.span,
    )
    .with_primary_label("variable being reassigned")
    .with_extra_label("same variable repeated on RHS", sub_left.span)
    .with_extra_label("operand could be compound-assigned", sub_right.span)
    .with_help(format!("Replace with: {var_text} {compound_op}"));

    let violation = match fix {
        Some(f) => violation.with_fix(f),
        None => violation,
    };

    Some(violation)
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| {
        check_for_compound_assignment(expr, ctx)
            .into_iter()
            .collect()
    })
}

pub const fn rule() -> Rule {
    Rule::new(
        "shorten_with_compound_assignment",
        "Compound assignment operators simplify simple arithmetic.",
        check,
        LintLevel::Hint,
    )
    .with_doc_url("https://www.nushell.sh/book/operators.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
