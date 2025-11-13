use std::collections::HashMap;

use nu_protocol::{
    Span, VarId,
    ast::{Expr, Expression, FindMapResult, PathMember},
};

use crate::{
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

/// Check if an expression accesses the `exit_code` field (either via `.exit_code` or `get exit_code`)
fn contains_exit_code_access(expr: &Expression, context: &LintContext) -> bool {
    use nu_protocol::ast::Traverse;

    expr.find_map(context.working_set, &|inner_expr| {
        // Check for field access like $var.exit_code
        if let Expr::FullCellPath(cell_path) = &inner_expr.expr
            && cell_path.tail.iter().any(|path_member| {
                matches!(
                    path_member,
                    PathMember::String { val, .. } if val == "exit_code"
                )
            })
        {
            log::debug!("Found .exit_code field access in assignment expression");
            return FindMapResult::Found(());
        }

        // Check for `get exit_code` command
        if let Expr::Call(call) = &inner_expr.expr {
            let decl_name = call.get_call_name(context);
            if decl_name == "get" {
                if let Some(arg) = call.get_positional_arg(0) {
                    // Check if the argument is a cell path with "exit_code"
                    if let Expr::CellPath(cell_path) = &arg.expr {
                        if cell_path.members.iter().any(|member| {
                            matches!(
                                member,
                                PathMember::String { val, .. } if val == "exit_code"
                            )
                        }) {
                            log::debug!("Found 'get exit_code' command in assignment expression");
                            return FindMapResult::Found(());
                        }
                    }
                    // Also check for string argument
                    else if let Expr::String(s) = &arg.expr {
                        if s == "exit_code" {
                            log::debug!("Found 'get exit_code' command in assignment expression");
                            return FindMapResult::Found(());
                        }
                    }
                }
            }
        }

        FindMapResult::Continue
    })
    .is_some()
}

fn extract_complete_assignment(
    expr: &Expression,
    context: &LintContext,
) -> Option<(VarId, String, Span, bool)> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    let decl_name = call.get_call_name(context);
    if !matches!(decl_name.as_str(), "let" | "mut") {
        return None;
    }

    let (var_id, var_name, _var_span) = call.extract_variable_declaration(context)?;

    let value_arg = call.get_positional_arg(1)?;

    if !assignment_has_complete(value_arg, context) {
        return None;
    }

    // Check if the assignment value itself accesses exit_code
    let exit_code_checked_in_assignment = contains_exit_code_access(value_arg, context);

    Some((var_id, var_name, expr.span, exit_code_checked_in_assignment))
}

/// Find variable assignments that store complete results
fn find_complete_assignments(context: &LintContext) -> HashMap<VarId, (String, Span, bool)> {
    use nu_protocol::ast::Traverse;

    let mut complete_assignments = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            extract_complete_assignment(expr, context)
                .into_iter()
                .collect()
        },
        &mut complete_assignments,
    );

    complete_assignments
        .into_iter()
        .map(|(id, name, span, exit_code_checked)| (id, (name, span, exit_code_checked)))
        .collect()
}

/// Check if an assignment value contains a complete command
fn assignment_has_complete(value_expr: &Expression, context: &LintContext) -> bool {
    use nu_protocol::ast::Traverse;

    value_expr
        .find_map(context.working_set, &|inner_expr| {
            if let Expr::Call(inner_call) = &inner_expr.expr {
                let inner_decl_name = inner_call.get_call_name(context);
                if inner_decl_name == "complete" {
                    return FindMapResult::Found(inner_call);
                }
            }
            FindMapResult::Continue
        })
        .is_some()
}

/// Find all exit code checks in the AST
fn find_exit_code_checks(context: &LintContext) -> HashMap<VarId, Span> {
    use nu_protocol::ast::Traverse;

    let mut exit_code_accesses = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            expr.extract_field_access("exit_code").into_iter().collect()
        },
        &mut exit_code_accesses,
    );

    exit_code_accesses.into_iter().collect()
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let variable_assignments = find_complete_assignments(context);
    let exit_code_checks = find_exit_code_checks(context);

    log::debug!("Found {} complete assignments", variable_assignments.len());
    for (var_id, (var_name, span, checked_in_assignment)) in &variable_assignments {
        log::debug!("  Complete assignment: var_id={var_id:?}, name={var_name}, span={span:?}, checked_in_assignment={checked_in_assignment}");
    }

    log::debug!("Found {} exit_code checks", exit_code_checks.len());
    for (var_id, span) in &exit_code_checks {
        log::debug!("  Exit code check: var_id={var_id:?}, span={span:?}");
    }

    variable_assignments
        .iter()
        .filter(|(var_id, (_, _, checked_in_assignment))| {
            // Skip if exit_code was checked in the assignment itself
            if *checked_in_assignment {
                log::debug!("Skipping var_id={var_id:?} - exit_code checked in assignment");
                return false;
            }
            // Skip if exit_code is checked via the variable later
            if exit_code_checks.contains_key(var_id) {
                log::debug!("Skipping var_id={var_id:?} - exit_code checked via variable");
                return false;
            }
            true
        })
        .map(|(_, (var_name, assignment_span, _))| {
            log::debug!("Creating violation for unchecked variable: {var_name}");
            RuleViolation::new_dynamic(
                "check_complete_exit_code",
                format!("External command result '{var_name}' stored but exit code not checked"),
                *assignment_span,
            )
            .with_suggestion_static(
                "Check 'exit_code' field to handle command failures: if $result.exit_code != 0 { \
                 ... }",
            )
        })
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "check_complete_exit_code",
        RuleCategory::ErrorHandling,
        Severity::Warning,
        "Check exit codes when using 'complete' to capture external command results",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
