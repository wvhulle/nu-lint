use nu_protocol::ast::Expr;

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

/// Get the loop variable name from an each call
fn get_each_loop_var_name(call: &nu_protocol::ast::Call, context: &LintContext) -> Option<String> {
    log::debug!(
        "get_each_loop_var_name: call has {} arguments",
        call.arguments.len()
    );
    let first_arg = call.arguments.first()?;

    let (nu_protocol::ast::Argument::Positional(block_expr)
    | nu_protocol::ast::Argument::Unknown(block_expr)) = first_arg
    else {
        log::debug!("First argument is not positional or unknown");
        return None;
    };

    log::debug!("First arg expr type: {:?}", block_expr.expr);

    let block_id = match &block_expr.expr {
        Expr::Block(id) | Expr::Closure(id) => *id,
        _ => {
            log::debug!("First arg is not a block or closure");
            return None;
        }
    };

    let block = context.working_set.get_block(block_id);
    log::debug!(
        "Block has {} required positional params",
        block.signature.required_positional.len()
    );
    log::debug!("Block signature: {:?}", block.signature);

    let var_id = block.signature.required_positional.first()?;

    var_id.var_id.map(|id| {
        let var = context.working_set.get_variable(id);
        let name = context.source[var.declaration_span.start..var.declaration_span.end].to_string();
        log::debug!("Loop var name extracted: {name}");
        name
    })
}

/// Check if an expression is just the loop variable (or property access on it)
fn is_loop_var_or_property(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    match &expr.expr {
        Expr::Var(var_id) => {
            let var = context.working_set.get_variable(*var_id);
            let var_name = &context.source[var.declaration_span.start..var.declaration_span.end];
            log::debug!("Checking var: {var_name} against loop var: {loop_var_name}");
            var_name == loop_var_name
        }
        Expr::FullCellPath(cell_path) => {
            // $x.property is also just filtering
            is_loop_var_or_property(&cell_path.head, context, loop_var_name)
        }
        _ => false,
    }
}

/// Check if a block contains side effects (print, save, mut, etc.)
fn has_side_effects(block_id: nu_protocol::BlockId, context: &LintContext) -> bool {
    let block = context.working_set.get_block(block_id);

    block
        .pipelines
        .iter()
        .flat_map(|p| &p.elements)
        .any(|elem| matches_side_effect_pattern(&elem.expr, context))
}

/// Check if an expression is a side-effect pattern
fn matches_side_effect_pattern(expr: &nu_protocol::ast::Expression, context: &LintContext) -> bool {
    match &expr.expr {
        Expr::Call(call) => {
            let decl_name = context.working_set.get_decl(call.decl_id).name();
            log::debug!("Found command in block: {decl_name}");

            // Side-effect commands
            if matches!(
                decl_name,
                "print" | "save" | "download" | "exit" | "mut" | "cd" | "source" | "use"
            ) {
                log::debug!("Found side effect command: {decl_name}");
                return true;
            }
            false
        }
        Expr::BinaryOp(_, op, _) => {
            // Check for variable assignment
            if matches!(
                op.expr,
                Expr::Operator(nu_protocol::ast::Operator::Assignment(_))
            ) {
                log::debug!("Found assignment operation");
                return true;
            }
            false
        }
        _ => false,
    }
}

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

    let decl_name = context.working_set.get_decl(call.decl_id).name();
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
    if has_side_effects(*then_block_id, context) {
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
    let is_loop_var = is_loop_var_or_property(&then_elem.expr, context, loop_var_name);

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

    let decl_name = context.working_set.get_decl(call.decl_id).name();
    if decl_name != "each" {
        return vec![];
    }

    log::debug!("Found 'each' call at span {:?}", expr.span);

    let Some(loop_var_name) = get_each_loop_var_name(call.as_ref(), context) else {
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
