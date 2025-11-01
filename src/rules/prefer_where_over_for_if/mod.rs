use nu_protocol::ast::Expr;
use std::collections::{HashMap, HashSet};

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

/// Check if a block contains only an if statement with append (filtering pattern)
fn is_filtering_only_pattern(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    let block = context.working_set.get_block(block_id);

    // Should have exactly one pipeline
    if block.pipelines.len() != 1 {
        return false;
    }

    let pipeline = &block.pipelines[0];

    // Should have exactly one element
    if pipeline.elements.len() != 1 {
        return false;
    }

    let elem = &pipeline.elements[0];

    // Must be an if statement
    let Expr::Call(call) = &elem.expr.expr else {
        return false;
    };

    let decl_name = context.working_set.get_decl(call.decl_id).name();
    if decl_name != "if" {
        return false;
    }

    // Get the then-block (should be third argument)
    let then_block_arg = call.arguments.get(2)?;
    let then_block_expr = match then_block_arg {
        nu_protocol::ast::Argument::Positional(e) | nu_protocol::ast::Argument::Unknown(e) => e,
        _ => return false,
    };

    let Expr::Block(then_block_id) = &then_block_expr.expr else {
        return false;
    };

    // Check if the then-block contains an append without transformation
    has_append_without_transformation(*then_block_id, context, loop_var_name)
}

/// Check if a block contains append of the loop variable without transformation
fn has_append_without_transformation(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    let block = context.working_set.get_block(block_id);

    for pipeline in &block.pipelines {
        for elem in &pipeline.elements {
            if let Expr::BinaryOp(_lhs, op, rhs) = &elem.expr.expr {
                if matches!(
                    op.expr,
                    Expr::Operator(nu_protocol::ast::Operator::Assignment(_))
                ) {
                    // Check if RHS contains append
                    if check_append_arg_is_loop_var(rhs, context, loop_var_name) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Check if an expression is an append call where the argument is just the loop variable
fn check_append_arg_is_loop_var(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    match &expr.expr {
        Expr::Call(call) => {
            let decl_name = context.working_set.get_decl(call.decl_id).name();
            if decl_name == "append" {
                // Check the argument
                if let Some(arg) = call.arguments.first() {
                    let arg_expr = match arg {
                        nu_protocol::ast::Argument::Positional(e)
                        | nu_protocol::ast::Argument::Unknown(e) => e,
                        _ => return false,
                    };

                    // Check if it's just the loop variable
                    if let Expr::Var(var_id) = &arg_expr.expr {
                        let var_name = context.working_set.get_variable(*var_id).name();
                        return var_name == loop_var_name;
                    } else if let Expr::FullCellPath(cell_path) = &arg_expr.expr {
                        // If it's a cell path like $x.field, that's still just filtering
                        // without transformation, so we can suggest where
                        if let Expr::Var(var_id) = &cell_path.head.expr {
                            let var_name = context.working_set.get_variable(*var_id).name();
                            return var_name == loop_var_name;
                        }
                    }
                }
            }
            false
        }
        Expr::FullCellPath(cell_path) => {
            check_append_arg_is_loop_var(&cell_path.head, context, loop_var_name)
        }
        Expr::Block(block_id) | Expr::Subexpression(block_id) => {
            let block = context.working_set.get_block(*block_id);
            block.pipelines.iter().any(|pipeline| {
                pipeline
                    .elements
                    .iter()
                    .any(|elem| check_append_arg_is_loop_var(&elem.expr, context, loop_var_name))
            })
        }
        _ => false,
    }
}

/// Get the loop variable name from a for loop
fn get_loop_var_name(call: &nu_protocol::ast::Call, context: &LintContext) -> Option<String> {
    let var_arg = call.arguments.first()?;
    let var_expr = match var_arg {
        nu_protocol::ast::Argument::Positional(e) | nu_protocol::ast::Argument::Unknown(e) => e,
        _ => return None,
    };

    if let Expr::Var(var_id) | Expr::VarDecl(var_id) = &var_expr.expr {
        Some(context.working_set.get_variable(*var_id).name().to_string())
    } else {
        None
    }
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    use nu_protocol::ast::Traverse;

    // Find all mut vars initialized to empty lists
    let mut empty_list_vars: HashMap<nu_protocol::VarId, (String, nu_protocol::Span)> =
        HashMap::new();

    context.ast.flat_map(
        context.working_set,
        &|expr| {
            let Expr::Call(call) = &expr.expr else {
                return Vec::new();
            };

            let decl_name = context.working_set.get_decl(call.decl_id).name();
            if decl_name != "mut" {
                return Vec::new();
            }

            let var_arg = call.arguments.first()?;
            let var_expr = match var_arg {
                nu_protocol::ast::Argument::Positional(e)
                | nu_protocol::ast::Argument::Unknown(e) => e,
                _ => return None,
            }?;

            let Expr::VarDecl(var_id) = &var_expr.expr else {
                return None;
            };

            // Check if initialized to empty list
            let init_arg = call.arguments.get(1)?;
            let init_expr = match init_arg {
                nu_protocol::ast::Argument::Positional(e)
                | nu_protocol::ast::Argument::Unknown(e) => e,
                _ => return None,
            }?;

            let is_empty_list = match &init_expr.expr {
                Expr::List(items) => items.is_empty(),
                Expr::Block(block_id) => {
                    let block = context.working_set.get_block(*block_id);
                    block
                        .pipelines
                        .first()
                        .and_then(|p| p.elements.first())
                        .is_some_and(
                            |elem| matches!(&elem.expr.expr, Expr::List(items) if items.is_empty()),
                        )
                }
                _ => false,
            };

            if is_empty_list {
                let var_name = &context.source[var_expr.span.start..var_expr.span.end];
                empty_list_vars.insert(*var_id, (var_name.to_string(), expr.span));
            }

            Vec::<()>::new()
        },
        &mut Vec::new(),
    );

    // Find vars used in filtering-only for loops
    let mut filtering_patterns: HashSet<nu_protocol::VarId> = HashSet::new();

    context.ast.flat_map(
        context.working_set,
        &|expr| {
            let Expr::Call(call) = &expr.expr else {
                return Vec::new();
            };

            let decl_name = context.working_set.get_decl(call.decl_id).name();
            if decl_name != "for" {
                return Vec::new();
            }

            // Get loop variable name
            let loop_var_name = get_loop_var_name(call, context)?;

            // Get the block (loop body) - last argument
            let block_arg = call.arguments.last()?;
            let block_expr = match block_arg {
                nu_protocol::ast::Argument::Positional(e)
                | nu_protocol::ast::Argument::Unknown(e) => e,
                _ => return None,
            }?;

            let Expr::Block(block_id) = &block_expr.expr else {
                return None;
            };

            // Check if this is a filtering-only pattern
            if is_filtering_only_pattern(*block_id, context, &loop_var_name) {
                // Find which variable is being accumulated
                let block = context.working_set.get_block(*block_id);
                for pipeline in &block.pipelines {
                    for elem in &pipeline.elements {
                        if let Expr::Call(if_call) = &elem.expr.expr {
                            let if_decl = context.working_set.get_decl(if_call.decl_id).name();
                            if if_decl == "if" {
                                // Find the assignment inside the if block
                                if let Some(
                                    nu_protocol::ast::Argument::Positional(then_expr)
                                    | nu_protocol::ast::Argument::Unknown(then_expr),
                                ) = if_call.arguments.get(2)
                                {
                                    if let Expr::Block(then_block_id) = &then_expr.expr {
                                        let then_block =
                                            context.working_set.get_block(*then_block_id);
                                        for p in &then_block.pipelines {
                                            for e in &p.elements {
                                                if let Expr::BinaryOp(lhs, op, _rhs) = &e.expr.expr
                                                {
                                                    if matches!(
                                                        op.expr,
                                                        Expr::Operator(
                                                            nu_protocol::ast::Operator::Assignment(
                                                                _
                                                            )
                                                        )
                                                    ) {
                                                        if let Expr::Var(var_id) = &lhs.expr {
                                                            filtering_patterns.insert(*var_id);
                                                        } else if let Expr::FullCellPath(
                                                            cell_path,
                                                        ) = &lhs.expr
                                                        {
                                                            if let Expr::Var(var_id) =
                                                                &cell_path.head.expr
                                                            {
                                                                filtering_patterns.insert(*var_id);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Vec::<()>::new()
        },
        &mut Vec::new(),
    );

    // Create violations
    let mut violations = Vec::new();
    for (var_id, (var_name, span)) in &empty_list_vars {
        if filtering_patterns.contains(var_id) {
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
