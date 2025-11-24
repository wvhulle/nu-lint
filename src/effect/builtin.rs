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
        Argument::Positional(expr) | Argument::Spread(expr) => {
            &context.source[expr.span.start..expr.span.end]
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

pub type BuiltinSideEffectPredicate = fn(&LintContext, &Call) -> bool;

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

fn is_unvalidated_variable(path: &str) -> bool {
    path.starts_with('$') && !path.starts_with("$in")
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

pub const BUILTIN_COMMAND_SIDE_EFFECTS: &[(
    &str,
    &[(BuiltinEffect, BuiltinSideEffectPredicate)],
)] = &[
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
            (BuiltinEffect::CommonEffect(CommonEffect::Dangerous), always),
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
        "error make",
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
    (
        "input list",
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
