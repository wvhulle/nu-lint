use nu_protocol::ast::{Argument, Call, ExternalArgument};

use crate::{ast::call::CallExt, context::LintContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SideEffect {
    Error,
    Dangerous,
    NoOutput,
    PipelineUnsafe,
    Print,
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

    let external_args: Vec<ExternalArgument> = call
        .arguments
        .iter()
        .filter_map(|arg| match arg {
            Argument::Positional(expr) => Some(ExternalArgument::Regular(expr.clone())),
            _ => None,
        })
        .collect();

    log::debug!(
        "Looking in registry for command '{command_name}' and side effect '{side_effect:?}'"
    );

    let result = COMMAND_SIDE_EFFECTS
        .iter()
        .find(|(name, _)| *name == command_name)
        .and_then(|(_, effects)| {
            effects
                .iter()
                .find(|(effect, _)| *effect == side_effect)
                .map(|(_, predicate)| {
                    log::debug!("Checking predicate for side effect '{side_effect:?}'");
                    predicate(context, call, &external_args)
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

pub fn can_error(command_name: &str, context: &LintContext, call: &Call) -> bool {
    has_side_effect(command_name, SideEffect::Error, context, call)
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

pub fn is_external_command_safe(command_name: &str) -> bool {
    COMMAND_SIDE_EFFECTS
        .iter()
        .find(|(name, _)| *name == command_name)
        .is_some_and(|(_, effects)| {
            !effects
                .iter()
                .any(|(effect, _)| *effect == SideEffect::PipelineUnsafe)
        })
}

pub fn external_command_has_no_output(command_name: &str) -> bool {
    COMMAND_SIDE_EFFECTS
        .iter()
        .find(|(name, _)| *name == command_name)
        .is_some_and(|(_, effects)| {
            effects
                .iter()
                .any(|(effect, _)| *effect == SideEffect::NoOutput)
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

pub fn has_recursive_flag(args: &[ExternalArgument], context: &LintContext) -> bool {
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

pub fn extract_arg_text<'a>(arg: &ExternalArgument, context: &'a LintContext) -> &'a str {
    match arg {
        ExternalArgument::Regular(expr) | ExternalArgument::Spread(expr) => {
            &context.source[expr.span.start..expr.span.end]
        }
    }
}

pub type SideEffectPredicate = fn(&LintContext, &Call, &[ExternalArgument]) -> bool;

const fn always(_context: &LintContext, _call: &Call, _args: &[ExternalArgument]) -> bool {
    true
}

fn prints_to_stdout(_context: &LintContext, call: &Call, _args: &[ExternalArgument]) -> bool {
    !call.has_named_flag("stderr")
}

fn io_category_can_error(context: &LintContext, call: &Call, _args: &[ExternalArgument]) -> bool {
    matches!(
        context
            .working_set
            .get_decl(call.decl_id)
            .signature()
            .category,
        nu_protocol::Category::Network | nu_protocol::Category::FileSystem
    )
}

fn rm_is_dangerous(context: &LintContext, _call: &Call, args: &[ExternalArgument]) -> bool {
    args.iter()
        .map(|arg| extract_arg_text(arg, context))
        .any(|path| is_dangerous_path(path) || is_unvalidated_variable(path))
        || has_recursive_flag(args, context)
}

fn mv_cp_is_dangerous(context: &LintContext, _call: &Call, args: &[ExternalArgument]) -> bool {
    args.iter()
        .map(|arg| extract_arg_text(arg, context))
        .any(|path| is_dangerous_path(path) || is_unvalidated_variable(path))
}

fn is_unvalidated_variable(path: &str) -> bool {
    path.starts_with('$') && !path.starts_with("$in")
}

const SYSTEM_DIRECTORIES: &[&str] = &[
    "/home", "/usr", "/etc", "/var", "/sys", "/proc", "/dev", "/boot", "/lib", "/bin", "/sbin",
];

const EXACT_DANGEROUS_PATHS: &[&str] = &["/", "~", "../", ".."];

const COMMAND_SIDE_EFFECTS: &[(&str, &[(SideEffect, SideEffectPredicate)])] = &[
    (
        "rm",
        &[
            (SideEffect::Error, always),
            (SideEffect::NoOutput, always),
            (SideEffect::Dangerous, rm_is_dangerous),
        ],
    ),
    (
        "mv",
        &[
            (SideEffect::Error, always),
            (SideEffect::NoOutput, always),
            (SideEffect::Dangerous, mv_cp_is_dangerous),
        ],
    ),
    (
        "cp",
        &[
            (SideEffect::Error, always),
            (SideEffect::Dangerous, mv_cp_is_dangerous),
        ],
    ),
    ("open", &[(SideEffect::Error, io_category_can_error)]),
    ("save", &[(SideEffect::Error, io_category_can_error)]),
    ("from", &[(SideEffect::Error, always)]),
    ("to", &[(SideEffect::Error, always)]),
    ("http get", &[(SideEffect::Error, always)]),
    ("http post", &[(SideEffect::Error, always)]),
    (
        "mkdir",
        &[(SideEffect::Error, always), (SideEffect::NoOutput, always)],
    ),
    (
        "touch",
        &[(SideEffect::Error, always), (SideEffect::NoOutput, always)],
    ),
    (
        "cd",
        &[(SideEffect::Error, always), (SideEffect::NoOutput, always)],
    ),
    ("sleep", &[(SideEffect::NoOutput, always)]),
    ("use", &[(SideEffect::NoOutput, always)]),
    ("hide", &[(SideEffect::NoOutput, always)]),
    (
        "source",
        &[(SideEffect::Error, always), (SideEffect::NoOutput, always)],
    ),
    (
        "source-env",
        &[(SideEffect::Error, always), (SideEffect::NoOutput, always)],
    ),
    ("exit", &[(SideEffect::NoOutput, always)]),
    (
        "error make",
        &[(SideEffect::Error, always), (SideEffect::NoOutput, always)],
    ),
    ("input", &[(SideEffect::Error, always)]),
    ("input list", &[(SideEffect::Error, always)]),
    (
        "print",
        &[
            (SideEffect::Print, prints_to_stdout),
            (SideEffect::NoOutput, always),
        ],
    ),
    ("curl", &[(SideEffect::PipelineUnsafe, always)]),
    ("wget", &[(SideEffect::PipelineUnsafe, always)]),
    ("find", &[(SideEffect::PipelineUnsafe, always)]),
    ("grep", &[(SideEffect::PipelineUnsafe, always)]),
    ("awk", &[(SideEffect::PipelineUnsafe, always)]),
    ("sed", &[(SideEffect::PipelineUnsafe, always)]),
    ("echo", &[]),
    ("printf", &[]),
    ("true", &[]),
    ("false", &[]),
    ("yes", &[]),
    ("date", &[]),
    ("pwd", &[]),
    ("whoami", &[]),
    ("ls", &[]),
    ("git", &[]),
    ("uname", &[]),
    ("arch", &[]),
    ("hostname", &[]),
    ("id", &[]),
    ("uptime", &[]),
    ("cal", &[]),
    ("basename", &[]),
    ("dirname", &[]),
    ("realpath", &[]),
    ("readlink", &[]),
    ("env", &[]),
    ("printenv", &[]),
    ("tr", &[]),
    ("cut", &[]),
    ("paste", &[]),
    ("column", &[]),
    ("fmt", &[]),
    ("fold", &[]),
    ("expand", &[]),
    ("unexpand", &[]),
    ("bc", &[]),
    ("dc", &[]),
    ("expr", &[]),
    ("mktemp", &[]),
];
