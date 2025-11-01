use nu_protocol::ast::Expr;

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

/// Get the loop variable name from a for loop  
fn get_loop_var_name(call: &nu_protocol::ast::Call, context: &LintContext) -> Option<String> {
    let var_arg = call.arguments.first()?;
    let (nu_protocol::ast::Argument::Positional(var_expr)
    | nu_protocol::ast::Argument::Unknown(var_expr)) = var_arg
    else {
        return None;
    };

    // Extract variable name from span
    let var_name = &context.source[var_expr.span.start..var_expr.span.end];
    log::debug!("Loop var name: {var_name}");
    Some(var_name.to_string())
}

/// Check if an expression is just the loop variable or property access on it
fn is_loop_var_reference(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    match &expr.expr {
        Expr::Var(var_id) => {
            let var = context.working_set.get_variable(*var_id);
            let var_name = &context.source[var.declaration_span.start..var.declaration_span.end];
            log::debug!("Checking var '{var_name}' against loop var '{loop_var_name}'");
            var_name == loop_var_name
        }
        Expr::FullCellPath(cell_path) => {
            // $x.property is also just the loop var
            is_loop_var_reference(&cell_path.head, context, loop_var_name)
        }
        _ => false,
    }
}

/// Check if an expression contains append of just the loop variable
fn contains_loop_var_append(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    match &expr.expr {
        Expr::Call(call) => {
            let decl_name = context.working_set.get_decl(call.decl_id).name();
            log::debug!("Found call to: {decl_name}");

            if decl_name == "append" {
                // Check the argument to append
                if let Some(
                    nu_protocol::ast::Argument::Positional(arg_expr)
                    | nu_protocol::ast::Argument::Unknown(arg_expr),
                ) = call.arguments.first()
                {
                    let is_loop_var = is_loop_var_reference(arg_expr, context, loop_var_name);
                    log::debug!("Append argument is loop var: {is_loop_var}");
                    return is_loop_var;
                }
            }
            false
        }
        Expr::FullCellPath(cell_path) => {
            contains_loop_var_append(&cell_path.head, context, loop_var_name)
        }
        Expr::Block(block_id) | Expr::Subexpression(block_id) | Expr::Closure(block_id) => {
            let block = context.working_set.get_block(*block_id);
            block
                .pipelines
                .iter()
                .flat_map(|p| &p.elements)
                .any(|elem| contains_loop_var_append(&elem.expr, context, loop_var_name))
        }
        _ => false,
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
        "Checking append pattern: block has {} pipelines",
        block.pipelines.len()
    );

    block
        .pipelines
        .iter()
        .flat_map(|p| &p.elements)
        .any(|elem| matches_append_assignment(&elem.expr, context, loop_var_name))
}

/// Check if an expression is an assignment with append
fn matches_append_assignment(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    let Expr::BinaryOp(_lhs, op, rhs) = &expr.expr else {
        return false;
    };

    if !matches!(
        op.expr,
        Expr::Operator(nu_protocol::ast::Operator::Assignment(_))
    ) {
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

    let then_block_id = match &then_block_expr.expr {
        Expr::Block(id) | Expr::Closure(id) => *id,
        _ => {
            log::debug!("Then-block is not a Block or Closure");
            return false;
        }
    };

    // Check if the then-block contains an append without transformation
    let result = has_append_without_transformation(then_block_id, context, loop_var_name);
    log::debug!("has_append_without_transformation: {result}");
    result
}

/// Check if a block contains just an empty list
fn is_empty_list_in_block(block_id: nu_protocol::BlockId, context: &LintContext) -> bool {
    log::debug!("Checking block for empty list");
    let block = context.working_set.get_block(block_id);

    let Some(pipeline) = block.pipelines.first() else {
        log::debug!("No pipelines in block");
        return false;
    };

    let Some(elem) = pipeline.elements.first() else {
        log::debug!("No elements in pipeline");
        return false;
    };

    match &elem.expr.expr {
        Expr::List(items) => {
            log::debug!("Found list in block with {} items", items.len());
            items.is_empty()
        }
        Expr::FullCellPath(cell_path) => {
            matches!(&cell_path.head.expr, Expr::List(items) if items.is_empty())
        }
        _ => {
            log::debug!("Block contains: {:?}", elem.expr.expr);
            false
        }
    }
}

/// Extract empty list variable declarations
fn extract_empty_list_vars(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Vec<(nu_protocol::VarId, String, nu_protocol::Span)> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };

    let decl_name = context.working_set.get_decl(call.decl_id).name();
    log::debug!("Checking call to: {decl_name}");

    if decl_name != "mut" {
        return vec![];
    }

    log::debug!("Found 'mut' declaration");

    let Some(var_arg) = call.arguments.first() else {
        log::debug!("No var argument");
        return vec![];
    };

    let (nu_protocol::ast::Argument::Positional(var_expr)
    | nu_protocol::ast::Argument::Unknown(var_expr)) = var_arg
    else {
        log::debug!("Var arg is not positional");
        return vec![];
    };

    log::debug!("Var expr type: {:?}", var_expr.expr);

    let Expr::VarDecl(var_id) = &var_expr.expr else {
        log::debug!("Not a VarDecl");
        return vec![];
    };

    // Check if initialized to empty list
    let Some(init_arg) = call.arguments.get(1) else {
        log::debug!("No init argument");
        return vec![];
    };

    let (nu_protocol::ast::Argument::Positional(init_expr)
    | nu_protocol::ast::Argument::Unknown(init_expr)) = init_arg
    else {
        log::debug!("Init arg is not positional");
        return vec![];
    };

    log::debug!("Init expr type: {:?}", init_expr.expr);

    let is_empty_list = match &init_expr.expr {
        Expr::List(items) => {
            log::debug!("Found list with {} items", items.len());
            items.is_empty()
        }
        Expr::Block(block_id) => is_empty_list_in_block(*block_id, context),
        _ => {
            log::debug!("Init expr is neither List nor Block: {:?}", init_expr.expr);
            false
        }
    };

    log::debug!("is_empty_list: {is_empty_list}");

    if is_empty_list {
        let var_name = &context.source[var_expr.span.start..var_expr.span.end];
        log::debug!("Found empty list var: {var_name} (id: {var_id:?})");
        vec![(*var_id, var_name.to_string(), expr.span)]
    } else {
        vec![]
    }
}

/// Extract variable ID from assignment LHS
fn extract_var_id_from_lhs(lhs: &nu_protocol::ast::Expression) -> Option<nu_protocol::VarId> {
    match &lhs.expr {
        Expr::Var(id) => {
            log::debug!("Found assigned var");
            Some(*id)
        }
        Expr::FullCellPath(cell_path) => {
            if let Expr::Var(id) = &cell_path.head.expr {
                log::debug!("Found assigned var via cell path");
                Some(*id)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Extract variable IDs assigned in an if statement's then block
fn extract_assigned_var_ids_from_if(
    if_call: &nu_protocol::ast::Call,
    context: &LintContext,
) -> Vec<nu_protocol::VarId> {
    let mut var_ids = Vec::new();

    let Some(
        nu_protocol::ast::Argument::Positional(then_expr)
        | nu_protocol::ast::Argument::Unknown(then_expr),
    ) = if_call.arguments.get(1)
    else {
        return var_ids;
    };

    let (Expr::Block(then_block_id) | Expr::Closure(then_block_id)) = &then_expr.expr else {
        return var_ids;
    };

    let then_block = context.working_set.get_block(*then_block_id);

    for p in &then_block.pipelines {
        for e in &p.elements {
            let Expr::BinaryOp(lhs, op, _rhs) = &e.expr.expr else {
                continue;
            };

            let is_assignment = matches!(
                op.expr,
                Expr::Operator(nu_protocol::ast::Operator::Assignment(_))
            );
            if !is_assignment {
                continue;
            }

            if let Some(id) = extract_var_id_from_lhs(lhs) {
                var_ids.push(id);
            }
        }
    }

    var_ids
}
/// Extract variables used in filtering for loops
fn extract_filtering_vars(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Vec<nu_protocol::VarId> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };

    let decl_name = context.working_set.get_decl(call.decl_id).name();
    if decl_name != "for" {
        return vec![];
    }

    log::debug!("Found 'for' loop");

    // Get loop variable name
    let Some(loop_var_name) = get_loop_var_name(call, context) else {
        log::debug!("Could not get loop var name");
        return vec![];
    };

    // Get the block (loop body) - last argument
    let Some(block_arg) = call.arguments.last() else {
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
            log::debug!("Loop body is not a block or closure");
            return vec![];
        }
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
    block: &nu_protocol::ast::Block,
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
        RuleCategory::Idioms,
        Severity::Warning,
        "Prefer 'where' filter over for loop with if statement",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
