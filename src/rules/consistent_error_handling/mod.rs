use std::collections::HashMap;

use nu_protocol::{Span, VarId, ast::Expr};

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn has_error_handling_in_pipeline(
    pipeline: &nu_protocol::ast::Pipeline,
    context: &LintContext,
) -> bool {
    use nu_protocol::ast::Traverse;

    let mut found_handling = Vec::new();

    // Check each element in the pipeline for error handling constructs
    for element in &pipeline.elements {
        element.expr.flat_map(
            context.working_set,
            &|expr| {
                if let Expr::Call(call) = &expr.expr {
                    let decl_name = context.working_set.get_decl(call.decl_id).name();
                    if decl_name == "try" || decl_name == "complete" {
                        return vec![true];
                    } else if decl_name == "do" {
                        // Check for -i flag in do command
                        for arg in &call.arguments {
                            if let nu_protocol::ast::Argument::Named(named) = arg
                                && named.0.item == "ignore"
                            {
                                return vec![true];
                            }
                        }
                    }
                }
                vec![]
            },
            &mut found_handling,
        );
    }

    !found_handling.is_empty()
}

fn has_external_command_in_pipeline(
    pipeline: &nu_protocol::ast::Pipeline,
    context: &LintContext,
) -> Option<Span> {
    use nu_protocol::ast::Traverse;

    for element in &pipeline.elements {
        let mut found_external = Vec::new();

        element.expr.flat_map(
            context.working_set,
            &|expr| {
                if let Expr::ExternalCall(_head, _args) = &expr.expr {
                    return vec![expr.span];
                }
                vec![]
            },
            &mut found_external,
        );

        if !found_external.is_empty() {
            return found_external.first().copied();
        }
    }

    None
}

/// Find variable assignments that store complete results
fn find_complete_assignments(context: &LintContext) -> HashMap<VarId, (String, Span)> {
    use nu_protocol::ast::Traverse;

    let mut complete_assignments = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            if let Expr::Call(call) = &expr.expr {
                let decl_name = context.working_set.get_decl(call.decl_id).name();
                if (decl_name == "let" || decl_name == "mut")
                    && let Some(var_arg) = call.arguments.first()
                    && let nu_protocol::ast::Argument::Positional(var_expr)
                    | nu_protocol::ast::Argument::Unknown(var_expr) = var_arg
                    && let Expr::VarDecl(var_id) = &var_expr.expr
                {
                    let var_name = &context.source[var_expr.span.start..var_expr.span.end];

                    if let Some(value_arg) = call.arguments.get(1)
                        && let nu_protocol::ast::Argument::Positional(value_expr)
                        | nu_protocol::ast::Argument::Unknown(value_expr) = value_arg
                        && assignment_has_complete(value_expr, context)
                    {
                        return vec![(*var_id, var_name.to_string(), expr.span)];
                    }
                }
            }
            vec![]
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

/// Check violations for unchecked complete assignments
fn check_unchecked_assignments(
    variable_assignments: &HashMap<VarId, (String, Span)>,
    exit_code_checks: &HashMap<VarId, Span>,
) -> Vec<RuleViolation> {
    variable_assignments
        .iter()
        .filter(|(var_id, _)| !exit_code_checks.contains_key(var_id))
        .map(|(_, (var_name, assignment_span))| {
            RuleViolation::new_dynamic(
                "consistent_error_handling",
                format!("External command result '{var_name}' stored but exit code not checked"),
                *assignment_span,
            )
            .with_suggestion_static(
                "Check 'exit_code' field to handle command failures: if $result.exit_code != 0 { ... }",
            )
        })
        .collect()
}

/// Check pipelines for external commands without error handling
fn check_pipeline_error_handling(context: &LintContext) -> Vec<RuleViolation> {
    context
        .ast
        .pipelines
        .iter()
        .filter(|pipeline| pipeline.elements.len() >= 2)
        .filter_map(|pipeline| {
            let has_error_handling = has_error_handling_in_pipeline(pipeline, context);
            let external_span = has_external_command_in_pipeline(pipeline, context)?;

            (!has_error_handling).then(|| {
                RuleViolation::new_static(
                    "consistent_error_handling",
                    "External command in pipeline without error handling - use 'complete' for error checking",
                    external_span,
                )
                .with_suggestion_static(
                    "Use 'complete' to capture exit codes: ^command | complete | if $in.exit_code != 0 { ... }",
                )
            })
        })
        .collect()
}

/// Find all external command calls in the AST
fn find_external_commands(context: &LintContext) -> Vec<Span> {
    use nu_protocol::ast::Traverse;

    let mut sequential_externals = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            if let Expr::ExternalCall(_head, _args) = &expr.expr {
                return vec![expr.span];
            }
            vec![]
        },
        &mut sequential_externals,
    );

    sequential_externals
}

/// Check if text contains error handling patterns
fn has_error_handling_between(text: &str) -> bool {
    text.contains("complete")
        || text.contains("try")
        || text.contains("&&")
        || text.contains("exit_code")
}

/// Check for sequential external commands without error handling
fn check_sequential_externals(context: &LintContext) -> Vec<RuleViolation> {
    const MAX_DISTANCE: usize = 100;

    let sequential_externals = find_external_commands(context);
    let mut violations = Vec::new();

    for i in 0..sequential_externals.len() {
        for j in (i + 1)..sequential_externals.len() {
            let first_span = sequential_externals[i];
            let second_span = sequential_externals[j];

            if second_span.start - first_span.end < MAX_DISTANCE {
                let between_text = &context.source[first_span.end..second_span.start];
                if !has_error_handling_between(between_text) {
                    let combined_span = nu_protocol::Span::new(first_span.start, second_span.end);
                    violations.push(
                        RuleViolation::new_static(
                            "consistent_error_handling",
                            "Sequential external commands without error checking - failures in first command ignored",
                            combined_span,
                        )
                        .with_suggestion_static(
                            "Check exit codes between commands or use '&&' for conditional execution",
                        ),
                    );
                }
            }
        }
    }

    violations
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let mut violations = Vec::new();

    // Find variable assignments with complete and exit code checks
    let variable_assignments = find_complete_assignments(context);
    let exit_code_checks = find_exit_code_checks(context);

    // Check for unchecked assignments
    violations.extend(check_unchecked_assignments(
        &variable_assignments,
        &exit_code_checks,
    ));

    // Check pipelines for error handling
    violations.extend(check_pipeline_error_handling(context));

    // Check sequential external commands
    violations.extend(check_sequential_externals(context));

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "consistent_error_handling",
        RuleCategory::ErrorHandling,
        Severity::Error,
        "Check external command results consistently for better error handling",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
