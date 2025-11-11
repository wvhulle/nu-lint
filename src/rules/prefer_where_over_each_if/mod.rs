use nu_protocol::ast::{Argument, Call, Expr, Expression};

use crate::{
    ast::{block::BlockExt, call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

/// Extract the then-block expression from an if call
fn get_if_then_block(call: &Call) -> Option<&Expression> {
    call.arguments.get(1).and_then(|arg| match arg {
        Argument::Positional(expr) | Argument::Unknown(expr) => Some(expr),
        _ => None,
    })
}

/// Check if then-block returns only the loop variable
fn then_block_returns_loop_var(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    let block = context.working_set.get_block(block_id);

    // Should have exactly one pipeline with one element that is the loop var
    block.pipelines.len() == 1
        && block.pipelines[0].elements.len() == 1
        && block.pipelines[0].elements[0]
            .expr
            .refers_to_variable(context, loop_var_name)
}

/// Check if block is a filtering pattern: `{ if <condition> { $loopvar } }`
fn is_filtering_pattern(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    let block = context.working_set.get_block(block_id);

    // Must have exactly one pipeline with one element (the if statement)
    if block.pipelines.len() != 1 || block.pipelines[0].elements.len() != 1 {
        return false;
    }

    let elem = &block.pipelines[0].elements[0];

    // Element must be an if call
    let Expr::Call(call) = &elem.expr.expr else {
        return false;
    };

    if call.get_call_name(context) != "if" {
        return false;
    }

    // Get the then-block and verify it has no side effects
    let Some(then_block_expr) = get_if_then_block(call) else {
        return false;
    };

    let Expr::Block(then_block_id) = &then_block_expr.expr else {
        return false;
    };

    // Then-block must have no side effects and return only the loop variable
    let then_block = context.working_set.get_block(*then_block_id);
    !then_block.has_side_effects()
        && then_block_returns_loop_var(*then_block_id, context, loop_var_name)
}

/// Extract block ID from each call's first argument
fn extract_each_block_id(call: &Call) -> Option<nu_protocol::BlockId> {
    call.arguments.first().and_then(|arg| match arg {
        Argument::Positional(expr) | Argument::Unknown(expr) => match &expr.expr {
            Expr::Block(id) | Expr::Closure(id) => Some(*id),
            _ => None,
        },
        _ => None,
    })
}

/// Check expression for the each-if pattern
fn check_expression(expr: &Expression, context: &LintContext) -> Vec<RuleViolation> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };

    if call.get_call_name(context) != "each" {
        return vec![];
    }

    let Some(loop_var_name) = call.loop_var_from_each(context) else {
        return vec![];
    };

    let Some(block_id) = extract_each_block_id(call) else {
        return vec![];
    };

    is_filtering_pattern(block_id, context, &loop_var_name)
        .then(|| {
            RuleViolation::new_static(
                "prefer_where_over_each_if",
                "Consider using 'where' for filtering instead of 'each' with 'if'",
                expr.span,
            )
            .with_suggestion_static("Use '$list | where <condition>' for better performance")
        })
        .into_iter()
        .collect()
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    use nu_protocol::ast::Traverse;

    let mut violations = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| check_expression(expr, context),
        &mut violations,
    );

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_where_over_each_if",
        RuleCategory::Idioms,
        Severity::Warning,
        "Use 'where' for filtering instead of 'each' with 'if'",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
