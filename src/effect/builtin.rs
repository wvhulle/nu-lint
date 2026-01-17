use nu_protocol::ast::{Argument, Call};

use crate::{
    context::LintContext,
    effect::{CommonEffect, is_dangerous_path},
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
        Argument::Positional(expr) | Argument::Spread(expr) => context.expr_text(expr),
        _ => "",
    }
}

pub fn can_error(command_name: &str, context: &LintContext, call: &Call) -> bool {
    has_builtin_side_effect(
        command_name,
        BuiltinEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
        context,
        call,
    )
}

pub type EffectWhenFlags = fn(&LintContext<'_>, &Call) -> bool;

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

fn ls_can_error(_context: &LintContext, call: &Call) -> bool {
    // ls without arguments lists current directory and rarely errors
    // ls with a path argument can error if the path doesn't exist
    call.arguments
        .iter()
        .any(|arg| matches!(arg, Argument::Positional(_)))
}

fn http_can_error(_context: &LintContext, call: &Call) -> bool {
    // http without a URL just shows help and doesn't error
    // http with a URL (positional arg) makes a network request that can fail
    call.arguments
        .iter()
        .any(|arg| matches!(arg, Argument::Positional(_)))
}

fn print_to_stdout(_context: &LintContext, call: &Call) -> bool {
    use crate::ast::call::CallExt;
    !call.has_named_flag("stderr")
}

fn rm_is_dangerous(context: &LintContext, call: &Call) -> bool {
    call.arguments
        .iter()
        .map(|arg| extract_arg_text(arg, context))
        .any(is_dangerous_path)
        || has_recursive_flag(call, context)
}

fn mv_cp_is_dangerous(context: &LintContext, call: &Call) -> bool {
    call.arguments
        .iter()
        .map(|arg| extract_arg_text(arg, context))
        .any(is_dangerous_path)
}

fn exit_is_dangerous(context: &LintContext, call: &Call) -> bool {
    call.arguments.iter().any(|arg| {
        if let Argument::Positional(expr) = arg {
            let code_text = context.expr_text(expr);
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
                BuiltinEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (
                BuiltinEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                rm_is_dangerous,
            ),
        ],
    ),
    (
        "mv",
        &[
            (
                BuiltinEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (
                BuiltinEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                mv_cp_is_dangerous,
            ),
        ],
    ),
    (
        "cp",
        &[
            (
                BuiltinEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (
                BuiltinEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                mv_cp_is_dangerous,
            ),
        ],
    ),
    (
        "open",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            io_category_can_error,
        )],
    ),
    (
        "save",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            io_category_can_error,
        )],
    ),
    (
        "from",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    // exit is not catchable by try - it terminates the process
    // Only flagged as Dangerous when exit code is non-zero
    (
        "exit",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
            exit_is_dangerous,
        )],
    ),
    (
        "http",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            http_can_error,
        )],
    ),
    (
        "mkdir",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "touch",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "cd",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    ("print", &[(BuiltinEffect::PrintToStdout, print_to_stdout)]),
    (
        "ls",
        &[(
            BuiltinEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            ls_can_error,
        )],
    ),
];
