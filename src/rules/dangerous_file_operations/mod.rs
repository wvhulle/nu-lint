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

const DANGEROUS_COMMANDS: &[&str] = &["rm", "mv", "cp"];

const SYSTEM_DIRECTORIES: &[&str] = &[
    "/home", "/usr", "/etc", "/var", "/sys", "/proc", "/dev", "/boot", "/lib", "/bin", "/sbin",
];

const EXACT_DANGEROUS_PATHS: &[&str] = &["/", "~", "../", ".."];

fn is_exact_dangerous_path(path: &str) -> bool {
    EXACT_DANGEROUS_PATHS.contains(&path)
}

fn is_root_wildcard_pattern(path: &str) -> bool {
    matches!(
        path,
        "/*" | "~/*"
            | "/home/*"
            | "/usr/*"
            | "/etc/*"
            | "/var/*"
            | "/sys/*"
            | "/proc/*"
            | "/dev/*"
            | "/boot/*"
            | "/lib/*"
            | "/bin/*"
            | "/sbin/*"
    )
}

fn is_system_directory(path: &str) -> bool {
    SYSTEM_DIRECTORIES.contains(&path) || path == "/dev/null"
}

fn is_system_subdirectory(path: &str) -> bool {
    if path.contains("/tmp/") {
        return false;
    }

    SYSTEM_DIRECTORIES
        .iter()
        .any(|dir| path.starts_with(&format!("{dir}/")))
}

fn is_shallow_home_path(path: &str) -> bool {
    if !path.starts_with("~.") && !path.starts_with("~/") {
        return false;
    }

    let after_tilde = &path[1..];
    let slash_count = after_tilde.matches('/').count();

    slash_count <= 1
}

fn is_dangerous_path(path_str: &str) -> bool {
    is_exact_dangerous_path(path_str)
        || is_root_wildcard_pattern(path_str)
        || is_system_directory(path_str)
        || is_system_subdirectory(path_str)
        || is_shallow_home_path(path_str)
        || path_str.starts_with("/..")
}

fn extract_arg_text<'a>(arg: &ExternalArgument, context: &'a LintContext) -> &'a str {
    match arg {
        ExternalArgument::Regular(expr) | ExternalArgument::Spread(expr) => {
            &context.source[expr.span.start..expr.span.end]
        }
    }
}

fn has_recursive_flag(args: &[ExternalArgument], context: &LintContext) -> bool {
    args.iter().any(|arg| {
        let arg_text = extract_arg_text(arg, context);
        matches!(
            arg_text,
            text if text.contains("-r")
                || text.contains("--recursive")
                || text.contains("-rf")
                || text.contains("-fr")
                || text.contains("--force")
        )
    })
}

fn extract_path_from_arg(arg: &ExternalArgument, context: &LintContext) -> String {
    extract_arg_text(arg, context).to_string()
}

fn is_if_block_containing(
    expr: &nu_protocol::ast::Expression,
    command_span: Span,
    context: &LintContext,
) -> bool {
    let Expr::Call(call) = &expr.expr else {
        return false;
    };

    call.is_call_to_command("if", context)
        && expr.span.start <= command_span.start
        && expr.span.end >= command_span.end
}

fn is_inside_if_block(context: &LintContext, command_span: Span) -> bool {
    use nu_protocol::ast::Traverse;

    let mut found_in_if = Vec::new();

    context.ast.flat_map(
        context.working_set,
        &|expr| {
            if is_if_block_containing(expr, command_span, context) {
                vec![true]
            } else {
                vec![]
            }
        },
        &mut found_in_if,
    );

    !found_in_if.is_empty()
}

fn is_dangerous_command(cmd_name: &str) -> bool {
    DANGEROUS_COMMANDS.contains(&cmd_name)
}

fn extract_dangerous_command(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Option<(Span, String, Vec<ExternalArgument>)> {
    match &expr.expr {
        Expr::ExternalCall(head, args) => {
            let cmd_name = &context.source[head.span.start..head.span.end];
            if is_dangerous_command(cmd_name) {
                Some((expr.span, cmd_name.to_string(), args.to_vec()))
            } else {
                None
            }
        }
        Expr::Call(call) => {
            let decl_name = call.get_call_name(context);
            if !is_dangerous_command(&decl_name) {
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

fn is_pipeline_variable(path: &str) -> bool {
    path.starts_with("$in")
}

fn is_unvalidated_variable(path: &str, command_span: Span, context: &LintContext) -> bool {
    path.starts_with('$')
        && !is_pipeline_variable(path)
        && !is_inside_if_block(context, command_span)
}

fn check_command_arguments(
    cmd_name: &str,
    args: &[ExternalArgument],
    command_span: Span,
    context: &LintContext,
    violations: &mut Vec<RuleViolation>,
) {
    let is_recursive = cmd_name == "rm" && has_recursive_flag(args, context);

    for arg in args {
        let path_str = extract_path_from_arg(arg, context);

        if is_dangerous_path(&path_str) {
            violations.push(create_dangerous_path_violation(
                cmd_name,
                &path_str,
                command_span,
                is_recursive,
            ));
        }

        if is_unvalidated_variable(&path_str, command_span, context) {
            violations.push(create_variable_validation_violation(
                cmd_name,
                &path_str,
                command_span,
            ));
        }
    }
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
        check_command_arguments(&cmd_name, &args, command_span, context, &mut violations);
    }

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "dangerous_file_operations",
        RuleCategory::ErrorHandling,
        Severity::Warning,
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
