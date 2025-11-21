use nu_protocol::{
    Span,
    ast::{Argument, Expr, Expression, ExternalArgument},
};

use crate::{
    ast::{
        call::CallExt,
        effect::{
            SideEffect, extract_arg_text, has_recursive_flag, has_side_effect, is_dangerous_path,
        },
    },
    context::LintContext,
    rule::Rule,
    violation::Violation,
};

fn extract_path_from_arg(arg: &ExternalArgument, context: &LintContext) -> String {
    extract_arg_text(arg, context).to_string()
}

fn is_if_block_containing(expr: &Expression, command_span: Span, context: &LintContext) -> bool {
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
                vec![()]
            } else {
                vec![]
            }
        },
        &mut found_in_if,
    );

    !found_in_if.is_empty()
}

fn extract_dangerous_command(
    expr: &Expression,
    context: &LintContext,
) -> Option<(Span, String, Vec<ExternalArgument>)> {
    match &expr.expr {
        Expr::ExternalCall(head, args) => {
            let cmd_name = &context.source[head.span.start..head.span.end];

            // For external commands, we only check known dangerous commands
            if !matches!(cmd_name, "rm" | "mv" | "cp") {
                return None;
            }

            Some((expr.span, cmd_name.to_string(), args.to_vec()))
        }
        Expr::Call(call) => {
            let decl_name = call.get_call_name(context);

            let external_args: Vec<ExternalArgument> = call
                .arguments
                .iter()
                .filter_map(|arg| match arg {
                    Argument::Positional(expr) => Some(ExternalArgument::Regular(expr.clone())),
                    _ => None,
                })
                .collect();

            if !has_side_effect(&decl_name, SideEffect::Dangerous, context, call) {
                return None;
            }

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
) -> Violation {
    let severity = if is_recursive { "CRITICAL" } else { "WARNING" };
    Violation::new(
        "dangerous_file_operations",
        format!(
            "{severity}: Dangerous file operation '{cmd_name} {path_str}' - could cause data loss"
        ),
        command_span,
    )
    .with_help(
        "Avoid operations on system paths. Use specific file paths and consider backup first",
    )
}

fn create_variable_validation_violation(
    cmd_name: &str,
    path_str: &str,
    command_span: Span,
) -> Violation {
    Violation::new(
        "dangerous_file_operations",
        format!("Variable '{path_str}' used in '{cmd_name}' command without visible validation"),
        command_span,
    )
    .with_help(format!(
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
    violations: &mut Vec<Violation>,
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

fn check(context: &LintContext) -> Vec<Violation> {
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

pub const fn rule() -> Rule {
    Rule::new(
        "dangerous_file_operations",
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
