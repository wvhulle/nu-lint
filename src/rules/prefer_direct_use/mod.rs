use nu_protocol::ast::Expr;
use std::collections::{HashMap, HashSet};

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

/// Check if an expression contains any transformation or filtering beyond direct copying
fn has_transformation_or_filter(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    let block = context.working_set.get_block(block_id);

    for pipeline in &block.pipelines {
        for elem in &pipeline.elements {
            // Check for if statements (filtering)
            if let Expr::Call(call) = &elem.expr.expr {
                let decl_name = context.working_set.get_decl(call.decl_id).name();
                if decl_name == "if" {
                    return true;
                }
            }

            // Check if the append uses the variable directly or with transformation
            if let Expr::BinaryOp(_lhs, op, rhs) = &elem.expr.expr {
                if matches!(
                    op.expr,
                    Expr::Operator(nu_protocol::ast::Operator::Assignment(_))
                ) {
                    // Check what's being appended
                    if has_transformation_in_append(rhs, context, loop_var_name) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Check if an append expression contains transformation beyond direct variable access
fn has_transformation_in_append(
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

                    // If it's just a variable reference to the loop variable, no transformation
                    if let Expr::Var(var_id) = &arg_expr.expr {
                        let var_name = context.working_set.get_variable(*var_id).name();
                        return var_name != loop_var_name;
                    } else if let Expr::FullCellPath(cell_path) = &arg_expr.expr {
                        if let Expr::Var(var_id) = &cell_path.head.expr {
                            let var_name = context.working_set.get_variable(*var_id).name();
                            // If it's $var.field or similar, that's a transformation
                            return var_name == loop_var_name && !cell_path.tail.is_empty();
                        }
                    }

                    // Any other expression is a transformation
                    return true;
                }
            }
        }
        Expr::FullCellPath(cell_path) => {
            return has_transformation_in_append(&cell_path.head, context, loop_var_name);
        }
        Expr::Block(block_id) | Expr::Subexpression(block_id) => {
            let block = context.working_set.get_block(*block_id);
            for pipeline in &block.pipelines {
                for elem in &pipeline.elements {
                    if has_transformation_in_append(&elem.expr, context, loop_var_name) {
                        return true;
                    }
                }
            }
        }
        _ => {}
    }
    false
}

/// Get the loop variable name from a for loop
fn get_loop_var_name(call: &nu_protocol::ast::Call, context: &LintContext) -> Option<String> {
    // First argument should be the loop variable
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

/// Check if the iteration source is a literal list
fn is_literal_list(expr: &nu_protocol::ast::Expression) -> bool {
    match &expr.expr {
        Expr::List(_) => true,
        Expr::Block(block_id) => {
            // Nushell parses [] as a block, check if it contains a list
            // This is a simplification - could be more thorough
            false
        }
        _ => false,
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

    // Find vars used in append operations within for loops
    let mut direct_copy_patterns: HashSet<nu_protocol::VarId> = HashSet::new();

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

            // Get the iterable (second argument)
            let iter_arg = call.arguments.get(1)?;
            let iter_expr = match iter_arg {
                nu_protocol::ast::Argument::Positional(e)
                | nu_protocol::ast::Argument::Unknown(e) => e,
                _ => return None,
            }?;

            let is_literal = is_literal_list(iter_expr);

            // Get the block (loop body)
            let block_arg = call.arguments.last()?;
            let block_expr = match block_arg {
                nu_protocol::ast::Argument::Positional(e)
                | nu_protocol::ast::Argument::Unknown(e) => e,
                _ => return None,
            }?;

            let Expr::Block(block_id) = &block_expr.expr else {
                return None;
            };

            // Check if this is a direct copy pattern (no transformation, no filtering)
            if !has_transformation_or_filter(*block_id, context, &loop_var_name) && is_literal {
                // Find which variable is being accumulated
                let block = context.working_set.get_block(*block_id);
                for pipeline in &block.pipelines {
                    for elem in &pipeline.elements {
                        if let Expr::BinaryOp(lhs, op, _rhs) = &elem.expr.expr {
                            if matches!(
                                op.expr,
                                Expr::Operator(nu_protocol::ast::Operator::Assignment(_))
                            ) {
                                if let Expr::Var(var_id) = &lhs.expr {
                                    direct_copy_patterns.insert(*var_id);
                                } else if let Expr::FullCellPath(cell_path) = &lhs.expr {
                                    if let Expr::Var(var_id) = &cell_path.head.expr {
                                        direct_copy_patterns.insert(*var_id);
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

    // Create violations for vars that are both empty list mut vars AND direct copy patterns
    let mut violations = Vec::new();
    for (var_id, (var_name, span)) in &empty_list_vars {
        if direct_copy_patterns.contains(var_id) {
            let violation = RuleViolation::new_dynamic(
                "prefer_direct_use",
                format!("Variable '{var_name}' is initialized as empty list and filled by copying items unchanged"),
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

pub fn rule() -> Rule {
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
