use std::collections::{HashMap, HashSet};

use nu_protocol::ast::Expr;

use crate::{
    ast::{BlockExt, CallExt, ExpressionExt},
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

/// Check if an element matches a transformation pattern
fn matches_transformation_pattern(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    match &expr.expr {
        Expr::Call(call) => call.is_call_to_command("if", context),
        Expr::BinaryOp(_lhs, op, rhs) => {
            matches!(
                op.expr,
                Expr::Operator(nu_protocol::ast::Operator::Assignment(_))
            ) && has_transformation_in_append(rhs, context, loop_var_name)
        }
        _ => false,
    }
}

/// Check if an expression contains any transformation or filtering beyond
/// direct copying
fn has_transformation_or_filter(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    block_id
        .all_elements(context)
        .iter()
        .any(|elem| matches_transformation_pattern(&elem.expr, context, loop_var_name))
}

fn has_transformation_in_append(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    match &expr.expr {
        Expr::Call(call) => {
            if call.is_call_to_command("append", context)
                && let Some(arg) = call.arguments.first()
            {
                let (nu_protocol::ast::Argument::Positional(arg_expr)
                | nu_protocol::ast::Argument::Unknown(arg_expr)) = arg
                else {
                    return false;
                };

                if let Expr::Var(_var_id) = &arg_expr.expr {
                    let var_name = arg_expr.span_text(context);
                    return var_name != loop_var_name;
                } else if let Expr::FullCellPath(cell_path) = &arg_expr.expr
                    && let Expr::Var(_var_id) = &cell_path.head.expr
                {
                    let var_name = cell_path.head.span_text(context);
                    return var_name == loop_var_name && !cell_path.tail.is_empty();
                }

                return true;
            }
        }
        Expr::FullCellPath(cell_path) => {
            return has_transformation_in_append(&cell_path.head, context, loop_var_name);
        }
        Expr::Block(block_id) | Expr::Subexpression(block_id) => {
            return block_id
                .all_elements(context)
                .iter()
                .any(|elem| has_transformation_in_append(&elem.expr, context, loop_var_name));
        }
        _ => {}
    }
    false
}

fn is_literal_list(expr: &nu_protocol::ast::Expression) -> bool {
    match &expr.expr {
        Expr::List(_) => true,
        Expr::FullCellPath(cell_path) => matches!(&cell_path.head.expr, Expr::List(_)),
        Expr::Keyword(keyword) => is_literal_list(&keyword.expr),
        _ => false,
    }
}

/// Extract variable IDs that are assigned to within a block (for append
/// detection)
fn extract_assigned_vars(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
) -> Vec<nu_protocol::VarId> {
    let mut var_ids = Vec::new();
    let block = context.working_set.get_block(block_id);
    for pipeline in &block.pipelines {
        for elem in &pipeline.elements {
            let Expr::BinaryOp(lhs, op, _rhs) = &elem.expr.expr else {
                continue;
            };

            if !matches!(
                op.expr,
                Expr::Operator(nu_protocol::ast::Operator::Assignment(_))
            ) {
                continue;
            }

            let var_id = match &lhs.expr {
                Expr::Var(var_id) => Some(*var_id),
                Expr::FullCellPath(cell_path) => match &cell_path.head.expr {
                    Expr::Var(var_id) => Some(*var_id),
                    _ => None,
                },
                _ => None,
            };

            if let Some(id) = var_id {
                var_ids.push(id);
            }
        }
    }
    var_ids
}

/// Create violations for variables that match the direct copy pattern
fn create_violations(
    empty_list_vars_map: &HashMap<nu_protocol::VarId, (String, nu_protocol::Span)>,
    direct_copy_set: &HashSet<nu_protocol::VarId>,
) -> Vec<RuleViolation> {
    let mut violations = Vec::new();
    for (var_id, (var_name, span)) in empty_list_vars_map {
        if direct_copy_set.contains(var_id) {
            let violation = RuleViolation::new_dynamic(
                "prefer_direct_use",
                format!(
                    "Variable '{var_name}' is initialized as empty list and filled by copying \
                     items unchanged"
                ),
                *span,
            )
            .with_suggestion_static(
                "Use the list directly instead of copying: 'let data = [1 2 3]'",
            );
            violations.push(violation);
        }
    }
    violations
}

/// Extract empty list variable declarations from mut statements
fn extract_empty_list_vars(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Vec<(nu_protocol::VarId, String, nu_protocol::Span)> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };

    if !call.is_call_to_command("mut", context) {
        return vec![];
    }

    let Some(var_arg) = call.arguments.first() else {
        return vec![];
    };

    let (nu_protocol::ast::Argument::Positional(var_expr)
    | nu_protocol::ast::Argument::Unknown(var_expr)) = var_arg
    else {
        return vec![];
    };

    let Expr::VarDecl(var_id) = &var_expr.expr else {
        return vec![];
    };

    // Check if initialized to empty list
    let Some(init_arg) = call.arguments.get(1) else {
        return vec![];
    };

    let (nu_protocol::ast::Argument::Positional(init_expr)
    | nu_protocol::ast::Argument::Unknown(init_expr)) = init_arg
    else {
        return vec![];
    };

    let is_empty_list = match &init_expr.expr {
        Expr::List(items) => {
            log::debug!("Found List with {} items", items.len());
            items.is_empty()
        }
        Expr::Block(block_id) => block_id.is_empty_list_block(context),
        _ => {
            log::debug!("Init expr is neither List nor Block: {:?}", init_expr.expr);
            false
        }
    };

    log::debug!("is_empty_list: {is_empty_list}");
    if is_empty_list {
        let var_name = var_expr.span_text(context);
        log::debug!("Found empty list var: {var_name} (id: {var_id:?})");
        vec![(*var_id, var_name.to_string(), expr.span)]
    } else {
        vec![]
    }
}

/// Extract direct copy patterns from for loops
fn extract_direct_copy_patterns(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Vec<nu_protocol::VarId> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };

    if !call.is_call_to_command("for", context) {
        return vec![];
    }

    let Some(loop_var_name) = call.loop_var_from_for(context) else {
        log::debug!("Could not get loop var name");
        return vec![];
    };

    log::debug!("Loop var name: {loop_var_name}");

    let Some(iter_arg) = call.arguments.get(1) else {
        log::debug!("No iterator argument");
        return vec![];
    };

    let (nu_protocol::ast::Argument::Positional(iter_expr)
    | nu_protocol::ast::Argument::Unknown(iter_expr)) = iter_arg
    else {
        return vec![];
    };

    let is_literal = is_literal_list(iter_expr);
    log::debug!(
        "Is literal list: {} (expr: {:?})",
        is_literal,
        iter_expr.expr
    );

    let Some(block_arg) = call.arguments.last() else {
        return vec![];
    };

    let (nu_protocol::ast::Argument::Positional(block_expr)
    | nu_protocol::ast::Argument::Unknown(block_expr)) = block_arg
    else {
        return vec![];
    };

    let Expr::Block(block_id) = &block_expr.expr else {
        return vec![];
    };

    if !has_transformation_or_filter(*block_id, context, &loop_var_name) && is_literal {
        log::debug!("Found direct copy pattern for loop var: {loop_var_name}");
        extract_assigned_vars(*block_id, context)
    } else {
        vec![]
    }
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    use nu_protocol::ast::Traverse;

    let mut empty_list_vars: Vec<(nu_protocol::VarId, String, nu_protocol::Span)> = Vec::new();

    context.ast.flat_map(
        context.working_set,
        &|expr| extract_empty_list_vars(expr, context),
        &mut empty_list_vars,
    );

    log::debug!("Total empty list vars found: {}", empty_list_vars.len());

    let empty_list_vars_map: HashMap<nu_protocol::VarId, (String, nu_protocol::Span)> =
        empty_list_vars
            .into_iter()
            .map(|(id, name, span)| (id, (name, span)))
            .collect();

    let mut direct_copy_patterns: Vec<nu_protocol::VarId> = Vec::new();

    context.ast.flat_map(
        context.working_set,
        &|expr| extract_direct_copy_patterns(expr, context),
        &mut direct_copy_patterns,
    );

    let direct_copy_set: HashSet<nu_protocol::VarId> = direct_copy_patterns.into_iter().collect();

    create_violations(&empty_list_vars_map, &direct_copy_set)
}

pub(crate) fn rule() -> Rule {
    Rule::new(
        "prefer_direct_use",
        RuleCategory::CodeQuality,
        Severity::Warning,
        "Prefer direct list use over copying items into a mutable list",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
