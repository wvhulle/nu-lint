use nu_protocol::{
    Span,
    ast::{Assignment, Expr, Expression, Math, Operator},
};

use crate::{
    LintLevel,
    ast::expression::ExpressionExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

fn refers_to_same_variable(expr1: &Expression, expr2: &Expression, context: &LintContext) -> bool {
    match (
        expr1.extract_variable_name(context),
        expr2.extract_variable_name(context),
    ) {
        (Some(name1), Some(name2)) => name1 == name2,
        _ => false,
    }
}

/// Semantic fix data: stores spans and operator needed to generate fix
pub struct FixData {
    full_span: Span,
    var_span: Span,
    right_operand_span: Span,
    compound_op: &'static str,
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

fn detect_compound_assignment(
    expr: &Expression,
    ctx: &LintContext,
) -> Option<(Detection, FixData)> {
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

    if !refers_to_same_variable(left, sub_left, ctx) {
        return None;
    }

    let compound_op = get_compound_operator(*operator)?;

    let var_text = ctx.expr_text(left);
    let op_symbol = get_operator_symbol(*operator);

    let violation = Detection::from_global_span(
        format!(
            "Use compound assignment: {var_text} {compound_op} instead of {var_text} = {var_text} \
             {op_symbol} ..."
        ),
        left.span,
    )
    .with_primary_label("variable being reassigned")
    .with_extra_label("same variable repeated on RHS", sub_left.span)
    .with_extra_label("operand could be compound-assigned", sub_right.span);

    let fix_data = FixData {
        full_span: expr.span,
        var_span: left.span,
        right_operand_span: sub_right.span,
        compound_op,
    };

    Some((violation, fix_data))
}

struct ShortenWithCompoundAssignment;

impl DetectFix for ShortenWithCompoundAssignment {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "compound_assignment"
    }

    fn short_description(&self) -> &'static str {
        "Compound assignment operators simplify simple arithmetic."
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/operators.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| {
            detect_compound_assignment(expr, ctx).into_iter().collect()
        })
    }

    fn fix(&self, ctx: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let var_text = ctx.span_text(fix_data.var_span);
        let right_text = ctx.span_text(fix_data.right_operand_span);
        let new_text = format!("{var_text} {} {right_text}", fix_data.compound_op);

        Some(Fix {
            explanation: format!("Replace with compound assignment: {new_text}").into(),
            replacements: vec![Replacement::new(fix_data.full_span, new_text)],
        })
    }
}

pub static RULE: &dyn Rule = &ShortenWithCompoundAssignment;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
