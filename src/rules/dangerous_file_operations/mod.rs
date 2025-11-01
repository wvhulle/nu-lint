use nu_protocol::{
    Span,
    ast::{Expr, ExternalArgument},
};

use crate::{
    ast::CallExt,
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

fn is_dangerous_path(path_str: &str) -> bool {
    path_str == "/"
        || path_str == "../"
        || path_str == ".."
        || path_str.starts_with("/..")
        || path_str.contains("/*")
        || path_str == "~"
        || path_str.starts_with("/home")
        || path_str.starts_with("/usr")
        || path_str.starts_with("/etc")
        || path_str.starts_with("/var")
        || path_str.starts_with("/sys")
        || path_str.starts_with("/proc")
}

fn has_recursive_flag(args: &[ExternalArgument], context: &LintContext) -> bool {
    args.iter().any(|arg| {
        let arg_text = match arg {
            ExternalArgument::Regular(expr) | ExternalArgument::Spread(expr) => {
                &context.source[expr.span.start..expr.span.end]
            }
        };

        arg_text.contains("-r")
            || arg_text.contains("--recursive")
            || arg_text.contains("-rf")
            || arg_text.contains("-fr")
            || arg_text.contains("--force")
    })
}

fn extract_path_from_arg(arg: &ExternalArgument, context: &LintContext) -> String {
    match arg {
        ExternalArgument::Regular(expr) | ExternalArgument::Spread(expr) => {
            context.source[expr.span.start..expr.span.end].to_string()
        }
    }
}

fn check_expr_for_if_block(
    expr: &nu_protocol::ast::Expression,
    command_span: Span,
    context: &LintContext,
) -> bool {
    let Expr::Call(call) = &expr.expr else {
        return false;
    };

    if !call.is_call_to_command("if", context) {
        return false;
    }

    expr.span.start <= command_span.start && expr.span.end >= command_span.end
}

fn is_inside_if_block(context: &LintContext, command_span: Span) -> bool {
    use nu_protocol::ast::Traverse;

    let mut found_in_if = Vec::new();

    context.ast.flat_map(
        context.working_set,
        &|expr| {
            if check_expr_for_if_block(expr, command_span, context) {
                vec![true]
            } else {
                vec![]
            }
        },
        &mut found_in_if,
    );

    !found_in_if.is_empty()
}

fn extract_dangerous_command(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Option<(Span, String, Vec<ExternalArgument>)> {
    match &expr.expr {
        Expr::ExternalCall(head, args) => {
            let cmd_name = &context.source[head.span.start..head.span.end];
            if cmd_name == "rm" || cmd_name == "mv" || cmd_name == "cp" {
                Some((expr.span, cmd_name.to_string(), args.to_vec()))
            } else {
                None
            }
        }
        Expr::Call(call) => {
            let decl_name = call.get_call_name(context);
            if decl_name != "rm" && decl_name != "mv" && decl_name != "cp" {
                return None;
            }

            let external_args: Vec<ExternalArgument> = call
                .arguments
                .iter()
                .filter_map(|arg| match arg {
                    nu_protocol::ast::Argument::Positional(expr) => {
                        Some(ExternalArgument::Regular(expr.clone()))
                    }
                    _ => None,
                })
                .collect();
            Some((expr.span, decl_name, external_args))
        }
        _ => None,
    }
}

fn create_dangerous_path_violation(
    cmd_name: &str,
    path_str: &str,
    command_span: Span,
    is_recursive: bool,
) -> RuleViolation {
    let severity = if is_recursive { "CRITICAL" } else { "WARNING" };
    RuleViolation::new_dynamic(
        "dangerous_file_operations",
        format!(
            "{severity}: Dangerous file operation '{cmd_name} {path_str}' - could cause data loss"
        ),
        command_span,
    )
    .with_suggestion_static(
        "Avoid operations on system paths. Use specific file paths and consider backup first",
    )
}

fn create_variable_validation_violation(
    cmd_name: &str,
    path_str: &str,
    command_span: Span,
) -> RuleViolation {
    RuleViolation::new_dynamic(
        "dangerous_file_operations",
        format!("Variable '{path_str}' used in '{cmd_name}' command without visible validation"),
        command_span,
    )
    .with_suggestion_dynamic(format!(
        "Validate variable before use: if ({path_str} | path exists) {{ {cmd_name} {path_str} }}"
    ))
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    use nu_protocol::ast::Traverse;

    let mut violations = Vec::new();

    let mut dangerous_commands = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            extract_dangerous_command(expr, context)
                .into_iter()
                .collect()
        },
        &mut dangerous_commands,
    );

    for (command_span, cmd_name, args) in dangerous_commands {
        let is_recursive = cmd_name == "rm" && has_recursive_flag(&args, context);

        for arg in &args {
            let path_str = extract_path_from_arg(arg, context);

            if is_dangerous_path(&path_str) {
                violations.push(create_dangerous_path_violation(
                    &cmd_name,
                    &path_str,
                    command_span,
                    is_recursive,
                ));
            }

            if path_str.starts_with('$') && !is_inside_if_block(context, command_span) {
                violations.push(create_variable_validation_violation(
                    &cmd_name,
                    &path_str,
                    command_span,
                ));
            }
        }
    }

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "dangerous_file_operations",
        RuleCategory::ErrorHandling,
        Severity::Error,
        "Detect dangerous file operations that could cause data loss",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
