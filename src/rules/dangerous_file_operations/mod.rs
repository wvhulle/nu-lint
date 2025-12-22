use nu_protocol::{
    Span,
    ast::{Call, Expr, Expression, ExternalArgument},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, span::SpanExt},
    context::LintContext,
    effect::{
        CommonEffect,
        builtin::{BuiltinEffect, extract_arg_text, has_builtin_side_effect, has_recursive_flag},
        external::{
            ExternEffect, extract_external_arg_text, has_external_recursive_flag,
            has_external_side_effect,
        },
        is_dangerous_path,
    },
    rule::Rule,
    violation::Violation,
};

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

enum DangerousCommand<'a> {
    External {
        span: Span,
        name: &'a str,
        args: &'a [ExternalArgument],
    },
    Builtin {
        span: Span,
        name: String,
        call: &'a Call,
    },
}

fn extract_dangerous_command<'a>(
    expr: &'a Expression,
    context: &'a LintContext,
) -> Option<DangerousCommand<'a>> {
    match &expr.expr {
        Expr::ExternalCall(head, args) => {
            let cmd_name = head.span.source_code(context);

            if !has_external_side_effect(
                cmd_name,
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                context,
                args,
            ) {
                return None;
            }

            Some(DangerousCommand::External {
                span: expr.span,
                name: cmd_name,
                args,
            })
        }
        Expr::Call(call) => {
            let decl_name = call.get_call_name(context);

            if !has_builtin_side_effect(
                &decl_name,
                BuiltinEffect::CommonEffect(CommonEffect::Dangerous),
                context,
                call,
            ) {
                return None;
            }

            Some(DangerousCommand::Builtin {
                span: expr.span,
                name: decl_name,
                call,
            })
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
    let label = if is_recursive {
        "recursive operation on dangerous path"
    } else {
        "dangerous path"
    };
    Violation::new(
        format!(
            "{severity}: Dangerous file operation '{cmd_name} {path_str}' - could cause data loss"
        ),
        command_span,
    )
    .with_primary_label(label)
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
        format!("Variable '{path_str}' used in '{cmd_name}' command without visible validation"),
        command_span,
    )
    .with_primary_label("unvalidated variable")
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

fn check_external_command(
    cmd_name: &str,
    args: &[ExternalArgument],
    command_span: Span,
    context: &LintContext,
    violations: &mut Vec<Violation>,
) {
    let is_recursive = cmd_name == "rm" && has_external_recursive_flag(args, context);

    for arg in args {
        let path_str = extract_external_arg_text(arg, context);

        if is_dangerous_path(path_str) {
            violations.push(create_dangerous_path_violation(
                cmd_name,
                path_str,
                command_span,
                is_recursive,
            ));
        }

        if is_unvalidated_variable(path_str, command_span, context) {
            violations.push(create_variable_validation_violation(
                cmd_name,
                path_str,
                command_span,
            ));
        }
    }
}

fn check_builtin_command(
    cmd_name: &str,
    call: &Call,
    command_span: Span,
    context: &LintContext,
    violations: &mut Vec<Violation>,
) {
    let is_recursive = cmd_name == "rm" && has_recursive_flag(call, context);

    for arg in &call.arguments {
        let path_str = extract_arg_text(arg, context);

        if is_dangerous_path(path_str) {
            violations.push(create_dangerous_path_violation(
                cmd_name,
                path_str,
                command_span,
                is_recursive,
            ));
        }

        if is_unvalidated_variable(path_str, command_span, context) {
            violations.push(create_variable_validation_violation(
                cmd_name,
                path_str,
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

    for cmd in dangerous_commands {
        match cmd {
            DangerousCommand::External { span, name, args } => {
                check_external_command(name, args, span, context, &mut violations);
            }
            DangerousCommand::Builtin { span, name, call } => {
                check_builtin_command(&name, call, span, context, &mut violations);
            }
        }
    }

    violations
}

pub const fn rule() -> Rule {
    Rule::new(
        "dangerous_file_operations",
        "Detect dangerous file operations that could cause data loss",
        check,
        LintLevel::Warning,
    )
    .with_doc_url("https://www.nushell.sh/book/running_externals.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
