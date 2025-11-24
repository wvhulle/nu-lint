use nu_protocol::ast::ExternalArgument;

use crate::{
    context::LintContext,
    effect::{CommonEffect, is_dangerous_path},
};

/// Things that may happen at runtime for external commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternEffect {
    /// Effect that is common between built-in and external commands.
    CommonEffect(CommonEffect),
    /// Silent, does not produce useful output
    NoDataInStdout,
    /// This command modifies the file system
    ModifiesFileSystem,
}

pub fn has_external_side_effect(
    command_name: &str,
    side_effect: ExternEffect,
    context: &LintContext,
    args: &[ExternalArgument],
) -> bool {
    log::debug!("Checking external side effect '{side_effect:?}' for command '{command_name}'");

    let result = EXTERNAL_COMMAND_SIDE_EFFECTS
        .iter()
        .find(|(name, _)| *name == command_name)
        .and_then(|(_, effects)| {
            effects
                .iter()
                .find(|(effect, _)| *effect == side_effect)
                .map(|(_, predicate)| {
                    log::debug!("Checking external predicate for side effect '{side_effect:?}'");
                    predicate(context, args)
                })
        })
        .unwrap_or(false);

    if result {
        log::debug!("External predicate matched for side effect '{side_effect:?}'");
    } else {
        log::debug!(
            "No matching external side effect '{side_effect:?}' found for command '{command_name}'"
        );
    }

    result
}

pub fn has_external_recursive_flag(args: &[ExternalArgument], context: &LintContext) -> bool {
    args.iter().any(|arg| {
        let arg_text = extract_external_arg_text(arg, context);
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

pub fn is_external_command_safe(command_name: &str) -> bool {
    EXTERNAL_COMMAND_SIDE_EFFECTS
        .iter()
        .find(|(name, _)| *name == command_name)
        .is_some_and(|(_, effects)| {
            !effects.iter().any(|(effect, _)| {
                *effect == ExternEffect::CommonEffect(CommonEffect::LikelyErrors)
            })
        })
}

pub fn external_command_has_no_output(command_name: &str) -> bool {
    EXTERNAL_COMMAND_SIDE_EFFECTS
        .iter()
        .find(|(name, _)| *name == command_name)
        .is_some_and(|(_, effects)| {
            effects
                .iter()
                .any(|(effect, _)| *effect == ExternEffect::NoDataInStdout)
        })
}

pub fn extract_external_arg_text<'a>(arg: &ExternalArgument, context: &'a LintContext) -> &'a str {
    match arg {
        ExternalArgument::Regular(expr) | ExternalArgument::Spread(expr) => {
            &context.source[expr.span.start..expr.span.end]
        }
    }
}

pub type ExternalSideEffectPredicate = fn(&LintContext, &[ExternalArgument]) -> bool;

const fn external_always(_context: &LintContext, _args: &[ExternalArgument]) -> bool {
    true
}

fn is_unvalidated_variable(path: &str) -> bool {
    path.starts_with('$')
}

fn external_rm_is_dangerous(context: &LintContext, args: &[ExternalArgument]) -> bool {
    args.iter()
        .map(|arg| extract_external_arg_text(arg, context))
        .any(|path| is_dangerous_path(path) || is_unvalidated_variable(path))
        || has_external_recursive_flag(args, context)
}

fn external_mv_cp_is_dangerous(context: &LintContext, args: &[ExternalArgument]) -> bool {
    args.iter()
        .map(|arg| extract_external_arg_text(arg, context))
        .any(|path| is_dangerous_path(path) || is_unvalidated_variable(path))
}

pub const EXTERNAL_COMMAND_SIDE_EFFECTS: &[(
    &str,
    &[(ExternEffect, ExternalSideEffectPredicate)],
)] = &[
    (
        "rm",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                external_always,
            ),
            (ExternEffect::ModifiesFileSystem, external_always),
            (ExternEffect::NoDataInStdout, external_always),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                external_rm_is_dangerous,
            ),
        ],
    ),
    (
        "mv",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                external_always,
            ),
            (ExternEffect::NoDataInStdout, external_always),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                external_mv_cp_is_dangerous,
            ),
            (ExternEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "cp",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                external_always,
            ),
            (ExternEffect::NoDataInStdout, external_always),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                external_mv_cp_is_dangerous,
            ),
            (ExternEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "tar",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                external_always,
            ),
            (ExternEffect::ModifiesFileSystem, external_always),
        ],
    ),
    ("echo", &[]),
    (
        "zip",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                external_always,
            ),
            (ExternEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "unzip",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                external_always,
            ),
            (ExternEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "rsync",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                external_always,
            ),
            (ExternEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "scp",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                external_always,
            ),
            (ExternEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "ssh",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                external_always,
            ),
            (ExternEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "curl",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                external_always,
            ),
            (ExternEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "wget",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                external_always,
            ),
            (ExternEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "find",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            external_always,
        )],
    ),
    (
        "grep",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            external_always,
        )],
    ),
    (
        "awk",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            external_always,
        )],
    ),
    (
        "sed",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                external_always,
            ),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                external_always,
            ),
        ],
    ),
    (
        "cat",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            external_always,
        )],
    ),
    (
        "head",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            external_always,
        )],
    ),
    (
        "tail",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            external_always,
        )],
    ),
    (
        "sort",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            external_always,
        )],
    ),
    (
        "uniq",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            external_always,
        )],
    ),
    (
        "wc",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            external_always,
        )],
    ),
    (
        "cut",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            external_always,
        )],
    ),
    (
        "xargs",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            external_always,
        )],
    ),
];
