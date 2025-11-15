use nu_protocol::ast::{Argument, Block, Call, Expr, Expression, Operator};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::Rule,
    violation::RuleViolation,
};

/// Check if an expression contains append of just the loop variable
fn contains_loop_var_append(expr: &Expression, context: &LintContext, loop_var_name: &str) -> bool {
    match &expr.expr {
        Expr::Call(call) => {
            let decl_name = call.get_call_name(context);
            log::debug!("Found call to: {decl_name}");

            if decl_name == "append" {
                // Check the argument to append
                if let Some(arg_expr) = call.get_first_positional_arg() {
                    let is_loop_var = arg_expr.refers_to_variable(context, loop_var_name);
                    log::debug!("Append argument is loop var: {is_loop_var}");
                    return is_loop_var;
                }
            }
            false
        }
        Expr::FullCellPath(cell_path) => {
            contains_loop_var_append(&cell_path.head, context, loop_var_name)
        }
        _ => expr.extract_block_id().is_some_and(|block_id| {
            let block = context.working_set.get_block(block_id);
            block
                .pipelines
                .iter()
                .flat_map(|p| &p.elements)
                .any(|elem| contains_loop_var_append(&elem.expr, context, loop_var_name))
        }),
    }
}

/// Check if a block contains only assignment with append of loop var
fn has_append_without_transformation(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    let block = context.working_set.get_block(block_id);
    log::debug!(
        "Checking append pattern: block has {} elements",
        block.all_elements().len()
    );

    block
        .all_elements()
        .iter()
        .any(|elem| matches_append_assignment(&elem.expr, context, loop_var_name))
}

/// Check if an expression is an assignment with append
fn matches_append_assignment(
    expr: &Expression,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    let Expr::BinaryOp(_lhs, op, rhs) = &expr.expr else {
        return false;
    };

    if !matches!(op.expr, Expr::Operator(Operator::Assignment(_))) {
        return false;
    }

    // Check if RHS contains append of loop var
    let result = contains_loop_var_append(rhs, context, loop_var_name);
    log::debug!("Assignment RHS contains loop var append: {result}");
    result
}

/// Check if a block contains only an if statement with append (filtering
/// pattern)
fn is_filtering_only_pattern(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    let block = context.working_set.get_block(block_id);
    log::debug!(
        "Checking filtering pattern: block has {} pipelines",
        block.pipelines.len()
    );

    // Should have exactly one pipeline with one element
    let Some(pipeline) = block.pipelines.first() else {
        log::debug!("Block has no pipelines");
        return false;
    };

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

    if !call.is_call_to_command("if", context) {
        log::debug!("Command is not 'if'");
        return false;
    }

    log::debug!("Found 'if' with {} arguments", call.arguments.len());

    // Must have exactly 2 arguments (condition, then-block) - no else clause
    if call.arguments.len() != 2 {
        log::debug!(
            "if statement has {} arguments, expected 2 (has else clause)",
            call.arguments.len()
        );
        return false;
    }

    // Get the then-block (second argument: condition, then-block)
    let Some(then_block_expr) = call.get_positional_arg(1) else {
        log::debug!("No then-block argument");
        return false;
    };

    let Some(then_block_id) = then_block_expr.extract_block_id() else {
        log::debug!("Then-block is not a Block or Closure");
        return false;
    };

    // Check if the then-block contains an append without transformation
    let result = has_append_without_transformation(then_block_id, context, loop_var_name);
    log::debug!("has_append_without_transformation: {result}");
    result
}

/// Extract empty list variable declarations
fn extract_empty_list_vars(
    expr: &Expression,
    context: &LintContext,
) -> Vec<(nu_protocol::VarId, String, nu_protocol::Span)> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };

    let decl_name = call.get_call_name(context);
    log::debug!("Checking call to: {decl_name}");

    if decl_name != "mut" {
        return vec![];
    }

    log::debug!("Found 'mut' declaration");

    let Some((var_id, var_name, _var_span)) = call.extract_variable_declaration(context) else {
        log::debug!("Could not extract variable declaration");
        return vec![];
    };

    // Check if initialized to empty list
    let Some(init_expr) = call.get_positional_arg(1) else {
        log::debug!("No init argument");
        return vec![];
    };

    log::debug!("Init expr type: {:?}", init_expr.expr);

    let is_empty_list = if init_expr.is_empty_list() {
        true
    } else if let Some(block_id) = init_expr.extract_block_id() {
        context
            .working_set
            .get_block(block_id)
            .is_empty_list_block()
    } else {
        false
    };

    log::debug!("is_empty_list: {is_empty_list}");

    if is_empty_list {
        log::debug!("Found empty list var: {var_name} (id: {var_id:?})");
        vec![(var_id, var_name, expr.span)]
    } else {
        vec![]
    }
}

/// Extract variable IDs assigned in an if statement's then block
fn extract_assigned_var_ids_from_if(
    if_call: &Call,
    context: &LintContext,
) -> Vec<nu_protocol::VarId> {
    let mut var_ids = Vec::new();

    let Some(Argument::Positional(then_expr) | Argument::Unknown(then_expr)) =
        if_call.arguments.get(1)
    else {
        return var_ids;
    };

    let (Expr::Block(then_block_id) | Expr::Closure(then_block_id)) = &then_expr.expr else {
        return var_ids;
    };

    let then_block = context.working_set.get_block(*then_block_id);

    for p in &then_block.pipelines {
        for e in &p.elements {
            let Expr::BinaryOp(_lhs, op, _rhs) = &e.expr.expr else {
                continue;
            };

            let is_assignment = matches!(op.expr, Expr::Operator(Operator::Assignment(_)));
            if !is_assignment {
                continue;
            }

            if let Some(id) = e.expr.extract_assigned_variable() {
                var_ids.push(id);
            }
        }
    }

    var_ids
}
/// Extract variables used in filtering for loops
fn extract_filtering_vars(expr: &Expression, context: &LintContext) -> Vec<nu_protocol::VarId> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };

    if !call.is_call_to_command("for", context) {
        return vec![];
    }

    log::debug!("Found 'for' loop");

    // Get loop variable name
    let Some(loop_var_name) = call.loop_var_from_for(context) else {
        log::debug!("Could not get loop var name");
        return vec![];
    };

    // Get the block (loop body) - last argument
    let Some(block_expr) = call.arguments.last().and_then(|arg| match arg {
        Argument::Positional(expr) | Argument::Unknown(expr) => Some(expr),
        _ => None,
    }) else {
        log::debug!("No block argument");
        return vec![];
    };

    let Some(block_id) = block_expr.extract_block_id() else {
        log::debug!("Loop body is not a block or closure");
        return vec![];
    };

    // Check if this is a filtering-only pattern
    if !is_filtering_only_pattern(block_id, context, &loop_var_name) {
        log::debug!("Not a filtering-only pattern");
        return vec![];
    }

    log::debug!("Found filtering pattern, extracting assigned variables");

    // Find which variable is being accumulated
    let block = context.working_set.get_block(block_id);
    extract_var_ids_from_if_statements(block, context)
}

/// Extract variable IDs from if statements in a block
fn extract_var_ids_from_if_statements(
    block: &Block,
    context: &LintContext,
) -> Vec<nu_protocol::VarId> {
    let mut var_ids = Vec::new();

    for pipeline in &block.pipelines {
        for elem in &pipeline.elements {
            let Expr::Call(if_call) = &elem.expr.expr else {
                continue;
            };

            let if_decl = context.working_set.get_decl(if_call.decl_id).name();
            if if_decl != "if" {
                continue;
            }

            var_ids.extend(extract_assigned_var_ids_from_if(if_call, context));
        }
    }

    var_ids
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    use std::collections::{HashMap, HashSet};

    use nu_protocol::ast::Traverse;

    // Find all mut vars initialized to empty lists
    let mut empty_list_vars = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| extract_empty_list_vars(expr, context),
        &mut empty_list_vars,
    );

    log::debug!("Found {} empty list vars", empty_list_vars.len());

    let empty_list_vars_map: HashMap<nu_protocol::VarId, (String, nu_protocol::Span)> =
        empty_list_vars
            .into_iter()
            .map(|(id, name, span)| (id, (name, span)))
            .collect();

    // Find vars used in filtering-only for loops
    let mut filtering_vars = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| extract_filtering_vars(expr, context),
        &mut filtering_vars,
    );

    log::debug!("Found {} filtering vars", filtering_vars.len());

    let filtering_set: HashSet<nu_protocol::VarId> = filtering_vars.into_iter().collect();

    // Create violations
    let mut violations = Vec::new();
    for (var_id, (var_name, span)) in &empty_list_vars_map {
        if filtering_set.contains(var_id) {
            log::debug!("Creating violation for var '{var_name}'");
            let violation = RuleViolation::new_dynamic(
                "prefer_where_over_for_if",
                format!("Variable '{var_name}' accumulates filtered items - use 'where' instead"),
                *span,
            )
            .with_suggestion_static(
                "Use '$input | where <condition>' for simple filtering without transformation",
            );
            violations.push(violation);
        }
    }

    log::debug!("Total violations: {}", violations.len());
    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_where_over_for_if",
        LintLevel::Warn,
        "Prefer 'where' filter over for loop with if statement",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
