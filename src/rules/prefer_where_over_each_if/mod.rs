use nu_protocol::ast::Expr;

use crate::{
    ast_utils::{BlockExt, CallExt, ExpressionExt},
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

/// Check if a block is a filtering pattern: if <condition> { $loopvar }
fn is_filtering_pattern(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    let block = context.working_set.get_block(block_id);
    log::debug!("Checking block with {} pipelines", block.pipelines.len());

    // Should have exactly one pipeline
    if block.pipelines.len() != 1 {
        log::debug!("Block has {} pipelines, expected 1", block.pipelines.len());
        return false;
    }

    let pipeline = &block.pipelines[0];
    log::debug!("Pipeline has {} elements", pipeline.elements.len());

    // Should have exactly one element (the if statement)
    if pipeline.elements.len() != 1 {
        log::debug!(
            "Pipeline has {} elements, expected 1",
            pipeline.elements.len()
        );
        return false;
    }

    let elem = &pipeline.elements[0];

    // Must be an if statement
    let Expr::Call(call) = &elem.expr.expr else {
        log::debug!("Element is not a Call");
        return false;
    };

    let decl_name = call.get_call_name(context);
    log::debug!("Found command: {decl_name}");

    if decl_name != "if" {
        log::debug!("Command is not 'if', it's '{decl_name}'");
        return false;
    }

    log::debug!(
        "Found 'if' statement with {} arguments",
        call.arguments.len()
    );

    // Get the then-block (should be second argument for if statement)
    // if <condition> { <then-block> }
    let Some(then_block_arg) = call.arguments.get(1) else {
        log::debug!("No then-block argument");
        return false;
    };

    let (nu_protocol::ast::Argument::Positional(then_block_expr)
    | nu_protocol::ast::Argument::Unknown(then_block_expr)) = then_block_arg
    else {
        log::debug!("Then-block is not positional");
        return false;
    };

    let Expr::Block(then_block_id) = &then_block_expr.expr else {
        log::debug!("Then-block is not a Block");
        return false;
    };

    log::debug!("Checking then-block for side effects");

    // Check for side effects
    if then_block_id.has_side_effects(context) {
        log::debug!("Then-block has side effects");
        return false;
    }

    // Check if the then-block just returns the loop variable
    let then_block = context.working_set.get_block(*then_block_id);

    log::debug!("Then-block has {} pipelines", then_block.pipelines.len());

    // The then-block should have one pipeline with one element that is the loop var
    if then_block.pipelines.len() != 1 {
        log::debug!(
            "Then-block has {} pipelines, expected 1",
            then_block.pipelines.len()
        );
        return false;
    }

    let then_pipeline = &then_block.pipelines[0];
    log::debug!(
        "Then-pipeline has {} elements",
        then_pipeline.elements.len()
    );

    if then_pipeline.elements.len() != 1 {
        log::debug!(
            "Then-pipeline has {} elements, expected 1",
            then_pipeline.elements.len()
        );
        return false;
    }

    let then_elem = &then_pipeline.elements[0];
    let is_loop_var = then_elem.expr.refers_to_variable(context, loop_var_name);

    log::debug!("Then-block returns loop var: {is_loop_var}");

    is_loop_var
}

/// Check each expression for the each-if pattern
fn check_expression(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Vec<RuleViolation> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };

    let decl_name = call.get_call_name(context);
    if decl_name != "each" {
        return vec![];
    }

    log::debug!("Found 'each' call at span {:?}", expr.span);

    let Some(loop_var_name) = call.loop_var_from_each(context) else {
        log::debug!("Could not get loop var name");
        return vec![];
    };

    log::debug!("Loop var name: {loop_var_name}");

    // Get the block argument
    let Some(block_arg) = call.arguments.first() else {
        log::debug!("No block argument");
        return vec![];
    };

    let (nu_protocol::ast::Argument::Positional(block_expr)
    | nu_protocol::ast::Argument::Unknown(block_expr)) = block_arg
    else {
        return vec![];
    };

    let block_id = match &block_expr.expr {
        Expr::Block(id) | Expr::Closure(id) => *id,
        _ => {
            log::debug!("Argument is not a block or closure");
            return vec![];
        }
    };

    if is_filtering_pattern(block_id, context, &loop_var_name) {
        log::debug!("Detected filtering pattern - creating violation");
        let violation = RuleViolation::new_dynamic(
            "prefer_where_over_each_if",
            "Consider using 'where' for filtering instead of 'each' with 'if'".to_string(),
            expr.span,
        )
        .with_suggestion_static("Use '$list | where <condition>' for better performance");
        vec![violation]
    } else {
        log::debug!("Not a filtering pattern");
        vec![]
    }
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    use nu_protocol::ast::Traverse;

    let mut violations = Vec::new();

    context.ast.flat_map(
        context.working_set,
        &|expr| check_expression(expr, context),
        &mut violations,
    );

    log::debug!("Total violations found: {}", violations.len());
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
