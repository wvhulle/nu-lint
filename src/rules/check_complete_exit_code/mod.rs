use std::collections::HashMap;

use nu_protocol::{Span, VarId, ast::Expr};

use crate::{
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

fn extract_complete_assignment(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Option<(VarId, String, Span)> {
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

    Some((var_id, var_name, expr.span))
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
                let inner_decl_name = inner_call.get_call_name(context);
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
        &|expr| expr.extract_field_access("exit_code").into_iter().collect(),
        &mut exit_code_accesses,
    );

    exit_code_accesses.into_iter().collect()
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
        Severity::Warning,
        "Check exit codes when using 'complete' to capture external command results",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
