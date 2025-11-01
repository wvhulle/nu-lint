use nu_protocol::ast::Expr;
use std::collections::{HashMap, HashSet};

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

/// Check if expression contains transformation (not just variable access)
fn has_transformation(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    match &expr.expr {
        Expr::Var(var_id) => {
            // Just a variable reference is not a transformation
            false
        }
        Expr::FullCellPath(cell_path) => {
            if let Expr::Var(var_id) = &cell_path.head.expr {
                let var_name = context.working_set.get_variable(*var_id).name();
                // Field access on the loop variable is a transformation
                var_name == loop_var_name && !cell_path.tail.is_empty()
            } else {
                true
            }
        }
        Expr::BinaryOp(_, _, _) | Expr::UnaryNot(_) => true,
        Expr::Call(_) => {
            // Any function call is a transformation
            true
        }
        Expr::Block(block_id) | Expr::Subexpression(block_id) => {
            let block = context.working_set.get_block(*block_id);
            block.pipelines.iter().any(|pipeline| {
                pipeline
                    .elements
                    .iter()
                    .any(|elem| has_transformation(&elem.expr, context, loop_var_name))
            })
        }
        _ => false,
    }
}

/// Check what's being appended to determine if there's transformation
fn check_append_has_transformation(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    match &expr.expr {
        Expr::Call(call) => {
            let decl_name = context.working_set.get_decl(call.decl_id).name();
            if decl_name == "append" {
                // Check the argument to append
                if let Some(arg) = call.arguments.first() {
                    let arg_expr = match arg {
                        nu_protocol::ast::Argument::Positional(e)
                        | nu_protocol::ast::Argument::Unknown(e) => e,
                        _ => return false,
                    };

                    return has_transformation(arg_expr, context, loop_var_name);
                }
            }
            false
        }
        Expr::FullCellPath(cell_path) => {
            check_append_has_transformation(&cell_path.head, context, loop_var_name)
        }
        Expr::Block(block_id) | Expr::Subexpression(block_id) => {
            let block = context.working_set.get_block(*block_id);
            block.pipelines.iter().any(|pipeline| {
                pipeline
                    .elements
                    .iter()
                    .any(|elem| check_append_has_transformation(&elem.expr, context, loop_var_name))
            })
        }
        _ => false,
    }
}

/// Check if a block contains transformation pattern
/// This includes:
/// 1. Simple transformation: for x in $input { $output = ($output | append ($x * 2)) }
/// 2. Filtering with transformation: for x in $input { if $x > 5 { $output = ($output | append ($x * 2)) } }
fn has_transformation_pattern(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    let block = context.working_set.get_block(block_id);

    for pipeline in &block.pipelines {
        for elem in &pipeline.elements {
            // Check direct assignment with transformation
            if let Expr::BinaryOp(_lhs, op, rhs) = &elem.expr.expr {
                if matches!(
                    op.expr,
                    Expr::Operator(nu_protocol::ast::Operator::Assignment(_))
                ) {
                    if check_append_has_transformation(rhs, context, loop_var_name) {
                        return true;
                    }
                }
            }

            // Check if inside an if statement
            if let Expr::Call(call) = &elem.expr.expr {
                let decl_name = context.working_set.get_decl(call.decl_id).name();
                if decl_name == "if" {
                    // Check the then-block (third argument)
                    if let Some(
                        nu_protocol::ast::Argument::Positional(then_expr)
                        | nu_protocol::ast::Argument::Unknown(then_expr),
                    ) = call.arguments.get(2)
                    {
                        if let Expr::Block(then_block_id) = &then_expr.expr {
                            let then_block = context.working_set.get_block(*then_block_id);
                            for p in &then_block.pipelines {
                                for e in &p.elements {
                                    if let Expr::BinaryOp(_lhs, op, rhs) = &e.expr.expr {
                                        if matches!(
                                            op.expr,
                                            Expr::Operator(nu_protocol::ast::Operator::Assignment(
                                                _
                                            ))
                                        ) {
                                            if check_append_has_transformation(
                                                rhs,
                                                context,
                                                loop_var_name,
                                            ) {
                                                return true;
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

    false
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

    // Find vars used in transformation patterns
    let mut transformation_patterns: HashSet<nu_protocol::VarId> = HashSet::new();

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

            // Check if this has transformation pattern
            if has_transformation_pattern(*block_id, context, &loop_var_name) {
                // Find which variable is being accumulated
                let block = context.working_set.get_block(*block_id);
                for pipeline in &block.pipelines {
                    for elem in &pipeline.elements {
                        // Check direct assignment
                        if let Expr::BinaryOp(lhs, op, _rhs) = &elem.expr.expr {
                            if matches!(
                                op.expr,
                                Expr::Operator(nu_protocol::ast::Operator::Assignment(_))
                            ) {
                                if let Expr::Var(var_id) = &lhs.expr {
                                    transformation_patterns.insert(*var_id);
                                } else if let Expr::FullCellPath(cell_path) = &lhs.expr {
                                    if let Expr::Var(var_id) = &cell_path.head.expr {
                                        transformation_patterns.insert(*var_id);
                                    }
                                }
                            }
                        }

                        // Check assignment inside if
                        if let Expr::Call(if_call) = &elem.expr.expr {
                            let if_decl = context.working_set.get_decl(if_call.decl_id).name();
                            if if_decl == "if" {
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
                                                            transformation_patterns.insert(*var_id);
                                                        } else if let Expr::FullCellPath(
                                                            cell_path,
                                                        ) = &lhs.expr
                                                        {
                                                            if let Expr::Var(var_id) =
                                                                &cell_path.head.expr
                                                            {
                                                                transformation_patterns
                                                                    .insert(*var_id);
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
        if transformation_patterns.contains(var_id) {
            let violation = RuleViolation::new_dynamic(
                "prefer_each_transformation",
                format!("Variable '{var_name}' accumulates transformed items - use 'each' instead"),
                *span,
            )
            .with_suggestion_static(
                "Use '$input | each {{ |x| transform }}' for simple transformation, or '$input | where <condition> | each {{ |x| transform }}' for filtering with transformation",
            );
            violations.push(violation);
        }
    }

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_each_transformation",
        RuleCategory::Idioms,
        Severity::Warning,
        "Prefer 'each' pipeline for transformations over mutable accumulation",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
