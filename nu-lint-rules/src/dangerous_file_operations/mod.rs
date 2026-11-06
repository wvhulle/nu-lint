use nu_lint_analysis::{
    LintLevel,
    ast::{call::CallExt, external::literal_content},
    effect::{
        CommonEffect,
        builtin::{BuiltinEffect, has_builtin_side_effect, has_recursive_flag, literal_arguments},
        external::{
            ExternEffect, has_external_recursive_flag, has_external_side_effect, literal_argument,
        },
        is_dangerous_path,
    },
    rule::{DetectFix, Rule},
    violation::Detection,
};
use nu_protocol::{
    Span,
    ast::{Call, Expr, Expression, ExternalArgument},
};

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
    context: &'a nu_lint_analysis::context::LintContext,
) -> Option<DangerousCommand<'a>> {
    match &expr.expr {
        Expr::ExternalCall(head, args) => {
            let cmd_name = literal_content(head)?;

            if !has_external_side_effect(
                head,
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
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
            if !has_builtin_side_effect(
                BuiltinEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                context,
                call,
            ) {
                return None;
            }

            Some(DangerousCommand::Builtin {
                span: expr.span,
                name: call.get_call_name(context),
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
) -> Detection {
    let severity = if is_recursive { "CRITICAL" } else { "WARNING" };
    let label = if is_recursive {
        "recursive operation on dangerous path"
    } else {
        "dangerous path"
    };
    Detection::from_global_span(
        format!(
            "{severity}: Dangerous file operation '{cmd_name} {path_str}' - could cause data loss"
        ),
        command_span,
    )
    .with_primary_label(label)
}

fn check_external_command(
    cmd_name: &str,
    args: &[ExternalArgument],
    command_span: Span,
    violations: &mut Vec<Detection>,
) {
    let is_recursive = cmd_name == "rm" && has_external_recursive_flag(args);

    for path_str in args.iter().filter_map(literal_argument) {
        if is_dangerous_path(path_str) {
            violations.push(create_dangerous_path_violation(
                cmd_name,
                path_str,
                command_span,
                is_recursive,
            ));
        }
    }
}

fn check_builtin_command(
    cmd_name: &str,
    call: &Call,
    command_span: Span,
    violations: &mut Vec<Detection>,
) {
    let is_recursive = cmd_name == "rm" && has_recursive_flag(call);

    for path_str in literal_arguments(call) {
        if is_dangerous_path(path_str) {
            violations.push(create_dangerous_path_violation(
                cmd_name,
                path_str,
                command_span,
                is_recursive,
            ));
        }
    }
}

struct DangerousFileOperations;

impl DetectFix for DangerousFileOperations {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "dangerous_file_operations"
    }

    fn short_description(&self) -> &'static str {
        "File operation on dangerous system path"
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(
        &self,
        context: &'a nu_lint_analysis::context::LintContext,
    ) -> Vec<(Detection, Self::FixInput<'a>)> {
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
                    check_external_command(name, args, span, &mut violations);
                }
                DangerousCommand::Builtin { span, name, call } => {
                    check_builtin_command(&name, call, span, &mut violations);
                }
            }
        }

        Self::no_fix(violations)
    }
}

pub static RULE: &dyn Rule = &DangerousFileOperations;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
