use nu_protocol::ast::Expr;

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn is_safe_external_command(cmd_name: &str) -> bool {
    matches!(cmd_name, "echo" | "cat" | "ls" | "pwd" | "whoami" | "date" | "true" | "false")
}

fn has_resource_management_in_pipeline(pipeline: &nu_protocol::ast::Pipeline, context: &LintContext) -> bool {
    use nu_protocol::ast::Traverse;

    let mut found_management = Vec::new();

    // Check each element in the pipeline for resource management constructs
    for element in &pipeline.elements {
        element.expr.flat_map(
            context.working_set,
            &|expr| {
                if let Expr::Call(call) = &expr.expr {
                    let decl_name = context.working_set.get_decl(call.decl_id).name();
                    if decl_name == "try" || decl_name == "complete" ||
                       decl_name == "collect" || decl_name == "with-env" ||
                       decl_name == "wrap" {
                        return vec![true];
                    } else if decl_name == "do" {
                        // Check for -i flag in do command
                        for arg in &call.arguments {
                            if let nu_protocol::ast::Argument::Named(named) = arg
                                && named.0.item == "ignore" {
                                    return vec![true];
                                }
                        }
                    }
                }
                vec![]
            },
            &mut found_management,
        );
    }

    !found_management.is_empty()
}

fn pipeline_ends_with_safe_operation(pipeline: &nu_protocol::ast::Pipeline, context: &LintContext) -> bool {
    if let Some(last_element) = pipeline.elements.last() {
        use nu_protocol::ast::Traverse;

        let mut found_safe_ending = Vec::new();

        last_element.expr.flat_map(
            context.working_set,
            &|expr| {
                if let Expr::Call(call) = &expr.expr {
                    let decl_name = context.working_set.get_decl(call.decl_id).name();
                    // Commands that properly consume/handle resources
                    if decl_name == "save" || decl_name == "to" ||
                       decl_name.starts_with("into ") {
                        return vec![true];
                    }
                }
                vec![]
            },
            &mut found_safe_ending,
        );

        !found_safe_ending.is_empty()
    } else {
        false
    }
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let mut violations = Vec::new();

    // Analyze each pipeline for resource management issues
    for pipeline in &context.ast.pipelines {
        // Skip single-element pipelines (no resource leak risk)
        if pipeline.elements.len() < 2 {
            continue;
        }


        // Check if the pipeline has proper resource management
        let has_resource_management = has_resource_management_in_pipeline(pipeline, context);
        let has_safe_ending = pipeline_ends_with_safe_operation(pipeline, context);

        // Skip if the pipeline already has proper resource management or safe ending
        if has_resource_management || has_safe_ending {
            continue;
        }

        // Check for specific resource-intensive operations
        for (element_idx, element) in pipeline.elements.iter().enumerate() {
            use nu_protocol::ast::Traverse;

            // Check for file operations
            let mut found_file_ops = Vec::new();
            element.expr.flat_map(
                context.working_set,
                &|expr| {
                    if let Expr::Call(call) = &expr.expr {
                        let decl_name = context.working_set.get_decl(call.decl_id).name();
                        if decl_name == "open" {
                            return vec![decl_name.to_string()];
                        }
                    }
                    vec![]
                },
                &mut found_file_ops,
            );

            // Check for network operations
            let mut found_network_ops = Vec::new();
            element.expr.flat_map(
                context.working_set,
                &|expr| {
                    if let Expr::Call(call) = &expr.expr {
                        let decl_name = context.working_set.get_decl(call.decl_id).name();
                        if decl_name == "http" || decl_name.starts_with("http ") ||
                           decl_name == "fetch" || decl_name == "connect" {
                            return vec![decl_name.to_string()];
                        }
                    }
                    vec![]
                },
                &mut found_network_ops,
            );

            // Check for external processes
            let mut found_external_processes = Vec::new();
            element.expr.flat_map(
                context.working_set,
                &|expr| {
                    if let Expr::ExternalCall(head, _args) = &expr.expr {
                        let cmd_name = &context.source[head.span.start..head.span.end];
                        return vec![cmd_name.to_string()];
                    }
                    vec![]
                },
                &mut found_external_processes,
            );

            // Generate violations for file operations
            if !found_file_ops.is_empty() && element_idx < pipeline.elements.len() - 1 {
                violations.push(
                    RuleViolation::new_static(
                        "resource_cleanup",
                        "File opened in pipeline without explicit resource management",
                        element.expr.span,
                    )
                    .with_suggestion_static(
                        "Consider using 'open file | collect' or handling the file operation in a single pipeline stage",
                    ),
                );
            }

            // Generate violations for network operations
            if !found_network_ops.is_empty() && element_idx < pipeline.elements.len() - 1 {
                violations.push(
                    RuleViolation::new_static(
                        "resource_cleanup",
                        "Network operation in pipeline without explicit resource management",
                        element.expr.span,
                    )
                    .with_suggestion_static(
                        "Consider using 'complete' to ensure proper connection handling",
                    ),
                );
            }

            // Generate violations for external processes (but skip safe commands)
            for cmd_name in found_external_processes {
                if !is_safe_external_command(&cmd_name) && element_idx < pipeline.elements.len() - 1 {
                    violations.push(
                        RuleViolation::new_dynamic(
                            "resource_cleanup",
                            format!("External process '{cmd_name}' in pipeline without explicit process management"),
                            element.expr.span,
                        )
                        .with_suggestion_static(
                            "Consider using 'complete' to ensure proper process handling and exit code checking",
                        ),
                    );
                }
            }
        }
    }

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "resource_cleanup",
        RuleCategory::ErrorHandling,
        Severity::Warning,
        "Ensure proper resource cleanup for files, connections, and processes",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;