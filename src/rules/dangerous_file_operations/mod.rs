use nu_protocol::ast::{Expr, ExternalArgument};

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn is_dangerous_path(path_str: &str) -> bool {
    // Check for dangerous patterns in file paths
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
    for arg in args {
        let arg_text = match arg {
            ExternalArgument::Regular(expr) | ExternalArgument::Spread(expr) => {
                &context.source[expr.span.start..expr.span.end]
            }
        };

        if arg_text.contains("-r")
            || arg_text.contains("--recursive")
            || arg_text.contains("-rf")
            || arg_text.contains("-fr")
            || arg_text.contains("--force")
        {
            return true;
        }
    }
    false
}

fn extract_path_from_arg(arg: &ExternalArgument, context: &LintContext) -> String {
    match arg {
        ExternalArgument::Regular(expr) | ExternalArgument::Spread(expr) => {
            context.source[expr.span.start..expr.span.end].to_string()
        }
    }
}

fn is_inside_if_block(context: &LintContext, command_span: nu_protocol::Span) -> bool {
    use nu_protocol::ast::Traverse;

    let mut found_in_if = Vec::new();

    context.ast.flat_map(
        context.working_set,
        &|expr| {
            if let Expr::Call(call) = &expr.expr {
                let decl_name = context.working_set.get_decl(call.decl_id).name();
                if decl_name == "if" {
                    // Check if the command span is within this if expression
                    if expr.span.start <= command_span.start && expr.span.end >= command_span.end {
                        return vec![true];
                    }
                }
            }
            vec![]
        },
        &mut found_in_if,
    );

    !found_in_if.is_empty()
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    use nu_protocol::ast::Traverse;

    let mut violations = Vec::new();

    // Find dangerous file operations
    let mut dangerous_commands = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            match &expr.expr {
                Expr::ExternalCall(head, args) => {
                    let cmd_name = &context.source[head.span.start..head.span.end];

                    // Check for dangerous file operations
                    if cmd_name == "rm" || cmd_name == "mv" || cmd_name == "cp" {
                        return vec![(expr.span, cmd_name.to_string(), args.clone())];
                    }
                }
                Expr::Call(call) => {
                    let decl_name = context.working_set.get_decl(call.decl_id).name();

                    // Check for built-in file operations
                    if decl_name == "rm" || decl_name == "mv" || decl_name == "cp" {
                        // Extract arguments for built-in commands
                        let mut external_args = Vec::new();
                        for arg in &call.arguments {
                            if let nu_protocol::ast::Argument::Positional(expr) = arg {
                                external_args.push(ExternalArgument::Regular(expr.clone()));
                            }
                        }
                        return vec![(
                            expr.span,
                            decl_name.to_string(),
                            external_args.into_boxed_slice(),
                        )];
                    }
                }
                _ => {}
            }
            vec![]
        },
        &mut dangerous_commands,
    );

    // Analyze each dangerous command
    for (command_span, cmd_name, args) in dangerous_commands {
        // Check for recursive flags on rm commands
        let is_recursive = cmd_name == "rm" && has_recursive_flag(&args, context);

        for arg in &args {
            let path_str = extract_path_from_arg(arg, context);

            // Check for dangerous paths
            if is_dangerous_path(&path_str) {
                let severity = if is_recursive { "CRITICAL" } else { "WARNING" };
                violations.push(
                    RuleViolation::new_dynamic(
                        "dangerous_file_operations",
                        format!(
                            "{severity}: Dangerous file operation '{cmd_name} {path_str}' - could \
                             cause data loss"
                        ),
                        command_span,
                    )
                    .with_suggestion_static(
                        "Avoid operations on system paths. Use specific file paths and consider \
                         backup first",
                    ),
                );
            }

            // Check for variables used without validation
            if path_str.starts_with('$') && !is_inside_if_block(context, command_span) {
                // Flag variable usage in dangerous commands as potentially risky
                violations.push(
                    RuleViolation::new_dynamic(
                        "dangerous_file_operations",
                        format!(
                            "Variable '{path_str}' used in '{cmd_name}' command without visible \
                             validation"
                        ),
                        command_span,
                    )
                    .with_suggestion_dynamic(format!(
                        "Validate variable before use: if ({path_str} | path exists) {{ \
                         {cmd_name} {path_str} }}"
                    )),
                );
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
