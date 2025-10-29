use std::collections::HashMap;

use nu_protocol::{Span, VarId, ast::Expr};

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn extract_complete_assignment(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Option<(VarId, String, Span)> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    let decl_name = context.working_set.get_decl(call.decl_id).name();
    if decl_name != "let" && decl_name != "mut" {
        return None;
    }

    let var_arg = call.arguments.first()?;

    let (nu_protocol::ast::Argument::Positional(var_expr)
    | nu_protocol::ast::Argument::Unknown(var_expr)) = var_arg
    else {
        return None;
    };

    let Expr::VarDecl(var_id) = &var_expr.expr else {
        return None;
    };

    let var_name = &context.source[var_expr.span.start..var_expr.span.end];

    let value_arg = call.arguments.get(1)?;

    let (nu_protocol::ast::Argument::Positional(value_expr)
    | nu_protocol::ast::Argument::Unknown(value_expr)) = value_arg
    else {
        return None;
    };

    if !assignment_has_complete(value_expr, context) {
        return None;
    }

    Some((*var_id, var_name.to_string(), expr.span))
}

/// Find variable assignments that store complete results
fn find_complete_assignments(context: &LintContext) -> HashMap<VarId, (String, Span)> {
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
        .map(|(id, name, span)| (id, (name, span)))
        .collect()
}

/// Check if an assignment value contains a complete command
fn assignment_has_complete(
    value_expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> bool {
    use nu_protocol::ast::Traverse;

    let mut has_complete = Vec::new();
    value_expr.flat_map(
        context.working_set,
        &|inner_expr| {
            if let Expr::Call(inner_call) = &inner_expr.expr {
                let inner_decl_name = context.working_set.get_decl(inner_call.decl_id).name();
                if inner_decl_name == "complete" {
                    return vec![true];
                }
            }
            vec![]
        },
        &mut has_complete,
    );

    !has_complete.is_empty()
}

/// Find all exit code checks in the AST
fn find_exit_code_checks(context: &LintContext) -> HashMap<VarId, Span> {
    use nu_protocol::ast::Traverse;

    let mut exit_code_accesses = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            if let Expr::FullCellPath(cell_path) = &expr.expr
                && let Expr::Var(var_id) = &cell_path.head.expr
                && accesses_exit_code_field(&cell_path.tail)
            {
                return vec![(*var_id, expr.span)];
            }
            vec![]
        },
        &mut exit_code_accesses,
    );

    exit_code_accesses.into_iter().collect()
}

/// Check if path accesses the `exit_code` field
fn accesses_exit_code_field(path_tail: &[nu_protocol::ast::PathMember]) -> bool {
    path_tail.iter().any(|path_member| {
        matches!(
            path_member,
            nu_protocol::ast::PathMember::String { val, .. } if val == "exit_code"
        )
    })
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let variable_assignments = find_complete_assignments(context);
    let exit_code_checks = find_exit_code_checks(context);

    variable_assignments
        .iter()
        .filter(|(var_id, _)| !exit_code_checks.contains_key(var_id))
        .map(|(_, (var_name, assignment_span))| {
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
        Severity::Error,
        "Check exit codes when using 'complete' to capture external command results",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
