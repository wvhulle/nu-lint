use nu_protocol::ast::{Argument, Call, ExternalArgument};

use crate::{ast::call::CallExt, context::LintContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SideEffect {
    MayErrorFrequently,
    Dangerous,
    NoUsefulOutput,
    PipelineUnsafe,
    Print,
    ModifiesFileSystem,
    UsesNetwork,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IoType {
    FileSystem,
    Network,
    Print,
}

pub fn has_side_effect(
    command_name: &str,
    side_effect: SideEffect,
    context: &LintContext,
    call: &Call,
) -> bool {
    log::debug!("Checking side effect '{side_effect:?}' for command '{command_name}'");
    log::debug!(
        "Looking in registry for command '{command_name}' and side effect '{side_effect:?}'"
    );

    let result = BUILTIN_COMMAND_SIDE_EFFECTS
        .iter()
        .find(|(name, _)| *name == command_name)
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

pub fn has_external_side_effect(
    command_name: &str,
    side_effect: SideEffect,
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

pub fn can_error(command_name: &str, context: &LintContext, call: &Call) -> bool {
    has_side_effect(command_name, SideEffect::MayErrorFrequently, context, call)
}

pub fn get_io_type(command_name: &str, context: &LintContext, call: &Call) -> Option<IoType> {
    if has_side_effect(command_name, SideEffect::Print, context, call) {
        return Some(IoType::Print);
    }

    if command_name.starts_with("http ") {
        return Some(IoType::Network);
    }

    let category = context
        .working_set
        .get_decl(call.decl_id)
        .signature()
        .category;

    match category {
        nu_protocol::Category::FileSystem => Some(IoType::FileSystem),
        nu_protocol::Category::Network => Some(IoType::Network),
        _ => None,
    }
}

pub fn get_external_io_type(command_name: &str) -> Option<IoType> {
    EXTERNAL_COMMAND_SIDE_EFFECTS
        .iter()
        .find(|(name, _)| *name == command_name)
        .and_then(|(_, effects)| {
            effects.iter().find_map(|(effect, _)| match effect {
                SideEffect::ModifiesFileSystem => Some(IoType::FileSystem),
                SideEffect::UsesNetwork => Some(IoType::Network),
                _ => None,
            })
        })
}

pub fn is_external_command_safe(command_name: &str) -> bool {
    BUILTIN_COMMAND_SIDE_EFFECTS
        .iter()
        .find(|(name, _)| *name == command_name)
        .is_some_and(|(_, effects)| {
            !effects
                .iter()
                .any(|(effect, _)| *effect == SideEffect::PipelineUnsafe)
        })
}

pub fn external_command_has_no_output(command_name: &str) -> bool {
    BUILTIN_COMMAND_SIDE_EFFECTS
        .iter()
        .find(|(name, _)| *name == command_name)
        .is_some_and(|(_, effects)| {
            effects
                .iter()
                .any(|(effect, _)| *effect == SideEffect::NoUsefulOutput)
        })
}

pub fn is_dangerous_path(path_str: &str) -> bool {
    EXACT_DANGEROUS_PATHS.contains(&path_str)
        || path_str.starts_with("/..")
        || matches!(
            path_str,
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
        || SYSTEM_DIRECTORIES.contains(&path_str)
        || path_str == "/dev/null"
        || (!path_str.contains("/tmp/")
            && SYSTEM_DIRECTORIES
                .iter()
                .any(|dir| path_str.starts_with(&format!("{dir}/"))))
        || ((path_str.starts_with("~.") || path_str.starts_with("~/"))
            && path_str[1..].matches('/').count() <= 1)
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

pub fn extract_arg_text<'a>(arg: &Argument, context: &'a LintContext) -> &'a str {
    match arg {
        Argument::Positional(expr) | Argument::Spread(expr) => {
            &context.source[expr.span.start..expr.span.end]
        }
        _ => "",
    }
}

pub fn extract_external_arg_text<'a>(arg: &ExternalArgument, context: &'a LintContext) -> &'a str {
    match arg {
        ExternalArgument::Regular(expr) | ExternalArgument::Spread(expr) => {
            &context.source[expr.span.start..expr.span.end]
        }
    }
}

pub type SideEffectPredicate = fn(&LintContext, &Call) -> bool;
pub type ExternalSideEffectPredicate = fn(&LintContext, &[ExternalArgument]) -> bool;

const fn always(_context: &LintContext, _call: &Call) -> bool {
    true
}

const fn external_always(_context: &LintContext, _args: &[ExternalArgument]) -> bool {
    true
}

fn prints_to_stdout(_context: &LintContext, call: &Call) -> bool {
    !call.has_named_flag("stderr")
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

fn rm_is_dangerous(context: &LintContext, call: &Call) -> bool {
    call.arguments
        .iter()
        .map(|arg| extract_arg_text(arg, context))
        .any(|path| is_dangerous_path(path) || is_unvalidated_variable(path))
        || has_recursive_flag(call, context)
}

fn external_rm_is_dangerous(context: &LintContext, args: &[ExternalArgument]) -> bool {
    args.iter()
        .map(|arg| extract_external_arg_text(arg, context))
        .any(|path| is_dangerous_path(path) || is_unvalidated_variable(path))
        || has_external_recursive_flag(args, context)
}

fn mv_cp_is_dangerous(context: &LintContext, call: &Call) -> bool {
    call.arguments
        .iter()
        .map(|arg| extract_arg_text(arg, context))
        .any(|path| is_dangerous_path(path) || is_unvalidated_variable(path))
}

fn external_mv_cp_is_dangerous(context: &LintContext, args: &[ExternalArgument]) -> bool {
    args.iter()
        .map(|arg| extract_external_arg_text(arg, context))
        .any(|path| is_dangerous_path(path) || is_unvalidated_variable(path))
}

fn is_unvalidated_variable(path: &str) -> bool {
    path.starts_with('$') && !path.starts_with("$in")
}

const SYSTEM_DIRECTORIES: &[&str] = &[
    "/home", "/usr", "/etc", "/var", "/sys", "/proc", "/dev", "/boot", "/lib", "/bin", "/sbin",
];

const EXACT_DANGEROUS_PATHS: &[&str] = &["/", "~", "../", ".."];

const BUILTIN_COMMAND_SIDE_EFFECTS: &[(&str, &[(SideEffect, SideEffectPredicate)])] = &[
    (
        "rm",
        &[
            (SideEffect::MayErrorFrequently, always),
            (SideEffect::NoUsefulOutput, always),
            (SideEffect::Dangerous, rm_is_dangerous),
            (SideEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "mv",
        &[
            (SideEffect::MayErrorFrequently, always),
            (SideEffect::NoUsefulOutput, always),
            (SideEffect::Dangerous, mv_cp_is_dangerous),
            (SideEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "cp",
        &[
            (SideEffect::MayErrorFrequently, always),
            (SideEffect::Dangerous, mv_cp_is_dangerous),
            (SideEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "open",
        &[
            (SideEffect::MayErrorFrequently, io_category_can_error),
            (SideEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "save",
        &[
            (SideEffect::MayErrorFrequently, io_category_can_error),
            (SideEffect::NoUsefulOutput, always),
            (SideEffect::ModifiesFileSystem, always),
        ],
    ),
    ("from", &[(SideEffect::MayErrorFrequently, always)]),
    ("to", &[(SideEffect::MayErrorFrequently, always)]),
    (
        "http get",
        &[
            (SideEffect::MayErrorFrequently, always),
            (SideEffect::UsesNetwork, always),
        ],
    ),
    (
        "http post",
        &[
            (SideEffect::MayErrorFrequently, always),
            (SideEffect::UsesNetwork, always),
        ],
    ),
    (
        "http put",
        &[
            (SideEffect::MayErrorFrequently, always),
            (SideEffect::UsesNetwork, always),
        ],
    ),
    (
        "http delete",
        &[
            (SideEffect::MayErrorFrequently, always),
            (SideEffect::UsesNetwork, always),
        ],
    ),
    (
        "http patch",
        &[
            (SideEffect::MayErrorFrequently, always),
            (SideEffect::UsesNetwork, always),
        ],
    ),
    (
        "http head",
        &[
            (SideEffect::MayErrorFrequently, always),
            (SideEffect::UsesNetwork, always),
        ],
    ),
    (
        "http options",
        &[
            (SideEffect::MayErrorFrequently, always),
            (SideEffect::UsesNetwork, always),
        ],
    ),
    (
        "mkdir",
        &[
            (SideEffect::MayErrorFrequently, always),
            (SideEffect::NoUsefulOutput, always),
            (SideEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "touch",
        &[
            (SideEffect::MayErrorFrequently, always),
            (SideEffect::NoUsefulOutput, always),
            (SideEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "cd",
        &[
            (SideEffect::MayErrorFrequently, always),
            (SideEffect::NoUsefulOutput, always),
        ],
    ),
    (
        "sleep",
        &[
            (SideEffect::MayErrorFrequently, always),
            (SideEffect::NoUsefulOutput, always),
        ],
    ),
    (
        "use",
        &[
            (SideEffect::MayErrorFrequently, always),
            (SideEffect::NoUsefulOutput, always),
        ],
    ),
    (
        "hide",
        &[
            (SideEffect::MayErrorFrequently, always),
            (SideEffect::NoUsefulOutput, always),
        ],
    ),
    (
        "source",
        &[
            (SideEffect::MayErrorFrequently, always),
            (SideEffect::NoUsefulOutput, always),
            (SideEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "source-env",
        &[
            (SideEffect::MayErrorFrequently, always),
            (SideEffect::NoUsefulOutput, always),
            (SideEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "load",
        &[
            (SideEffect::MayErrorFrequently, always),
            (SideEffect::ModifiesFileSystem, always),
        ],
    ),
    ("exit", &[(SideEffect::NoUsefulOutput, always)]),
    (
        "error make",
        &[
            (SideEffect::MayErrorFrequently, always),
            (SideEffect::NoUsefulOutput, always),
        ],
    ),
    ("input", &[(SideEffect::MayErrorFrequently, always)]),
    ("input list", &[(SideEffect::MayErrorFrequently, always)]),
    (
        "print",
        &[
            (SideEffect::Print, prints_to_stdout),
            (SideEffect::NoUsefulOutput, always),
        ],
    ),
    (
        "echo",
        &[
            (SideEffect::Print, always),
            (SideEffect::NoUsefulOutput, always),
        ],
    ),
    ("ls", &[]),
    ("git", &[]),
];

const EXTERNAL_COMMAND_SIDE_EFFECTS: &[(&str, &[(SideEffect, ExternalSideEffectPredicate)])] = &[
    (
        "rm",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::NoUsefulOutput, external_always),
            (SideEffect::Dangerous, external_rm_is_dangerous),
            (SideEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "mv",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::NoUsefulOutput, external_always),
            (SideEffect::Dangerous, external_mv_cp_is_dangerous),
            (SideEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "cp",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::Dangerous, external_mv_cp_is_dangerous),
            (SideEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "tar",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "zip",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "unzip",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "rsync",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "scp",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::ModifiesFileSystem, external_always),
            (SideEffect::UsesNetwork, external_always),
        ],
    ),
    (
        "ssh",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::UsesNetwork, external_always),
        ],
    ),
    (
        "curl",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::PipelineUnsafe, external_always),
            (SideEffect::ModifiesFileSystem, external_always),
            (SideEffect::UsesNetwork, external_always),
        ],
    ),
    (
        "wget",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::PipelineUnsafe, external_always),
            (SideEffect::ModifiesFileSystem, external_always),
            (SideEffect::UsesNetwork, external_always),
        ],
    ),
    (
        "find",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::PipelineUnsafe, external_always),
        ],
    ),
    (
        "grep",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::PipelineUnsafe, external_always),
        ],
    ),
    (
        "awk",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::PipelineUnsafe, external_always),
        ],
    ),
    (
        "sed",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::PipelineUnsafe, external_always),
        ],
    ),
    (
        "cat",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "head",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "tail",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "sort",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "uniq",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "wc",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::ModifiesFileSystem, external_always),
        ],
    ),
    (
        "cut",
        &[
            (SideEffect::MayErrorFrequently, external_always),
            (SideEffect::ModifiesFileSystem, external_always),
        ],
    ),
    ("tr", &[(SideEffect::MayErrorFrequently, external_always)]),
    (
        "xargs",
        &[(SideEffect::MayErrorFrequently, external_always)],
    ),
];
