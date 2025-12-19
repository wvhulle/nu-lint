use nu_protocol::ast::{Argument, Call};

use crate::{
    context::LintContext,
    effect::{CommonEffect, is_dangerous_path, is_unvalidated_variable},
};
/// Things that may happen at runtime for built-in Nu commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinEffect {
    /// Effect for a built-in command that is common with external commands
    CommonEffect(CommonEffect),
    /// Builtin command prints to standard output in terminal
    PrintToStdout,
}

pub fn has_builtin_side_effect(
    command_name: &str,
    side_effect: BuiltinEffect,
    context: &LintContext,
    call: &Call,
) -> bool {
    log::debug!("Checking side effect '{side_effect:?}' for command '{command_name}'");
    log::debug!(
        "Looking in registry for command '{command_name}' and side effect '{side_effect:?}'"
    );

    let result = BUILTIN_COMMAND_SIDE_EFFECTS
        .iter()
        .find(|(name, _)| *name == command_name || command_name.starts_with(&format!("{name} ")))
        .and_then(|(_, effects)| {
            effects
                .iter()
                .find(|(effect, _)| *effect == side_effect)
                .map(|(_, predicate)| {
                    log::debug!("Checking predicate for side effect '{side_effect:?}'");
                    predicate(context, call)
                })
        })
        .unwrap_or(false);

    if result {
        log::debug!("Predicate matched for side effect '{side_effect:?}'");
    } else {
        log::debug!("No matching side effect '{side_effect:?}' found for command '{command_name}'");
    }

    result
}

pub fn has_recursive_flag(call: &Call, context: &LintContext) -> bool {
    call.arguments.iter().any(|arg| {
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

pub fn extract_arg_text<'a>(arg: &Argument, context: &'a LintContext) -> &'a str {
    match arg {
        Argument::Positional(expr) | Argument::Spread(expr) => {
            std::str::from_utf8(context.working_set.get_span_contents(expr.span)).unwrap_or("")
        }
        _ => "",
    }
}

pub fn can_error(command_name: &str, context: &LintContext, call: &Call) -> bool {
    has_builtin_side_effect(
        command_name,
        BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
        context,
        call,
    )
}

pub type EffectWhenFlags = fn(&LintContext, &Call) -> bool;

const fn always(_context: &LintContext, _call: &Call) -> bool {
    true
}

fn io_category_can_error(context: &LintContext, call: &Call) -> bool {
    matches!(
        context
            .working_set
            .get_decl(call.decl_id)
            .signature()
            .category,
        nu_protocol::Category::Network | nu_protocol::Category::FileSystem
    )
}

fn print_to_stdout(_context: &LintContext, call: &Call) -> bool {
    use crate::ast::call::CallExt;
    !call.has_named_flag("stderr")
}

fn rm_is_dangerous(context: &LintContext, call: &Call) -> bool {
    call.arguments
        .iter()
        .map(|arg| extract_arg_text(arg, context))
        .any(|path| is_dangerous_path(path) || is_unvalidated_variable(path))
        || has_recursive_flag(call, context)
}

fn mv_cp_is_dangerous(context: &LintContext, call: &Call) -> bool {
    call.arguments
        .iter()
        .map(|arg| extract_arg_text(arg, context))
        .any(|path| is_dangerous_path(path) || is_unvalidated_variable(path))
}

fn exit_is_dangerous(context: &LintContext, call: &Call) -> bool {
    call.arguments.iter().any(|arg| {
        if let Argument::Positional(expr) = arg {
            let code_text =
                std::str::from_utf8(context.working_set.get_span_contents(expr.span)).unwrap_or("");
            code_text != "0" && !code_text.starts_with('$')
        } else {
            false
        }
    })
}

pub const BUILTIN_COMMAND_SIDE_EFFECTS: &[(&str, &[(BuiltinEffect, EffectWhenFlags)])] = &[
    (
        "rm",
        &[
            (
                BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (
                BuiltinEffect::CommonEffect(CommonEffect::Dangerous),
                rm_is_dangerous,
            ),
        ],
    ),
    (
        "mv",
        &[
            (
                BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (
                BuiltinEffect::CommonEffect(CommonEffect::Dangerous),
                mv_cp_is_dangerous,
            ),
        ],
    ),
    (
        "cp",
        &[
            (
                BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (
                BuiltinEffect::CommonEffect(CommonEffect::Dangerous),
                mv_cp_is_dangerous,
            ),
        ],
    ),
    (
        "open",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
            io_category_can_error,
        )],
    ),
    (
        "save",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
            io_category_can_error,
        )],
    ),
    (
        "from",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "exit",
        &[
            (
                BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (
                BuiltinEffect::CommonEffect(CommonEffect::Dangerous),
                exit_is_dangerous,
            ),
        ],
    ),
    (
        "error",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "to",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "http",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "mkdir",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "touch",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "cd",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "sleep",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "hide",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "source",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "source-env",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "load",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "input",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    ("print", &[(BuiltinEffect::PrintToStdout, print_to_stdout)]),
    (
        "ls",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    ("git", &[]),
];

#[cfg(test)]
mod tests {
    use nu_protocol::ast::Expr;

    use super::*;

    fn with_builtin_call<F, R>(source: &str, f: F) -> R
    where
        F: for<'b> FnOnce(&LintContext<'b>, &Call) -> R,
    {
        LintContext::test_with_parsed_source(source, |context| {
            let call = context
                .ast
                .pipelines
                .first()
                .and_then(|pipeline| pipeline.elements.first())
                .and_then(|element| match &element.expr.expr {
                    Expr::Call(call) => Some(call),
                    _ => None,
                })
                .expect("Expected a call expression");
            f(&context, call)
        })
    }

    #[test]
    fn test_exit_zero_is_not_dangerous() {
        with_builtin_call("exit 0", |context, call| {
            assert!(
                !has_builtin_side_effect(
                    "exit",
                    BuiltinEffect::CommonEffect(CommonEffect::Dangerous),
                    context,
                    call
                ),
                "exit 0 should not be dangerous"
            );
        });
    }

    #[test]
    fn test_exit_nonzero_is_dangerous() {
        with_builtin_call("exit 1", |context, call| {
            assert!(
                has_builtin_side_effect(
                    "exit",
                    BuiltinEffect::CommonEffect(CommonEffect::Dangerous),
                    context,
                    call
                ),
                "exit 1 should be dangerous"
            );
        });
    }

    #[test]
    fn test_exit_variable_is_not_dangerous() {
        with_builtin_call("exit $status", |context, call| {
            assert!(
                !has_builtin_side_effect(
                    "exit",
                    BuiltinEffect::CommonEffect(CommonEffect::Dangerous),
                    context,
                    call
                ),
                "exit $status should not be marked dangerous (unknown at lint time)"
            );
        });
    }

    #[test]
    fn test_print_without_stderr_prints_to_stdout() {
        with_builtin_call("print 'hello'", |context, call| {
            assert!(
                has_builtin_side_effect("print", BuiltinEffect::PrintToStdout, context, call),
                "print should print to stdout by default"
            );
        });
    }

    #[test]
    fn test_print_with_stderr_does_not_print_to_stdout() {
        with_builtin_call("print --stderr 'hello'", |context, call| {
            assert!(
                !has_builtin_side_effect("print", BuiltinEffect::PrintToStdout, context, call),
                "print --stderr should not print to stdout"
            );
        });
    }
}
