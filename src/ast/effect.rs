use std::{collections::HashMap, sync::LazyLock};

use nu_protocol::ast::{Argument, Call, ExternalArgument};

use crate::{ast::call::CallExt, context::LintContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IoType {
    FileSystem,
    Network,
    Print,
}

/// Side effects that cannot be derived from `nu_protocol::Category`
/// These represent behavioral aspects not captured by Nushell's built-in
/// categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SideEffect {
    /// Can fail or throw errors (requires error handling)
    Error,
    /// Potentially destructive operations (dangerous file operations)
    Dangerous,
    /// Commands that produce no useful output for pipelines
    NoOutput,
    /// External commands that need complete wrapper in pipelines
    PipelineUnsafe,
    /// Specifically prints to stdout (subset of System category)
    Print,
}

// ============================================================================
// ENHANCED PREDICATE-BASED SIDE EFFECTS SYSTEM
// ============================================================================

/// Context for evaluating side effects that depend on command arguments
#[derive(Debug)]
pub struct CommandContext<'a> {
    pub call: &'a Call,
    pub args: &'a [ExternalArgument],
}

/// Predicate function that determines if a command has a side effect based on
/// context
pub type SideEffectPredicate = fn(&str, &LintContext, &CommandContext) -> bool;

/// Enhanced command metadata with context-dependent predicates
#[derive(Debug)]
pub struct CommandPredicates {
    /// All side effects with their predicates
    pub effects: Vec<(SideEffect, SideEffectPredicate)>,
}

impl CommandPredicates {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    pub fn add(mut self, effect: SideEffect, predicate: SideEffectPredicate) -> Self {
        self.effects.push((effect, predicate));
        self
    }
}

pub const SYSTEM_DIRECTORIES: &[&str] = &[
    "/home", "/usr", "/etc", "/var", "/sys", "/proc", "/dev", "/boot", "/lib", "/bin", "/sbin",
];

pub const EXACT_DANGEROUS_PATHS: &[&str] = &["/", "~", "../", ".."];

pub fn extract_arg_text<'a>(arg: &ExternalArgument, context: &'a LintContext) -> &'a str {
    match arg {
        ExternalArgument::Regular(expr) | ExternalArgument::Spread(expr) => {
            &context.source[expr.span.start..expr.span.end]
        }
    }
}

pub fn is_exact_dangerous_path(path: &str) -> bool {
    EXACT_DANGEROUS_PATHS.contains(&path)
}

pub fn is_root_wildcard_pattern(path: &str) -> bool {
    matches!(
        path,
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
}

pub fn is_system_directory(path: &str) -> bool {
    SYSTEM_DIRECTORIES.contains(&path) || path == "/dev/null"
}

pub fn is_system_subdirectory(path: &str) -> bool {
    if path.contains("/tmp/") {
        return false;
    }

    SYSTEM_DIRECTORIES
        .iter()
        .any(|dir| path.starts_with(&format!("{dir}/")))
}

pub fn is_shallow_home_path(path: &str) -> bool {
    if !path.starts_with("~.") && !path.starts_with("~/") {
        return false;
    }

    let after_tilde = &path[1..];
    let slash_count = after_tilde.matches('/').count();

    slash_count <= 1
}

pub fn is_dangerous_path(path_str: &str) -> bool {
    is_exact_dangerous_path(path_str)
        || is_root_wildcard_pattern(path_str)
        || is_system_directory(path_str)
        || is_system_subdirectory(path_str)
        || is_shallow_home_path(path_str)
        || path_str.starts_with("/..")
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

pub fn is_unvalidated_variable(path: &str) -> bool {
    path.starts_with('$') && !path.starts_with("$in")
}

/// Check if rm command has dangerous arguments
pub fn rm_is_dangerous(_cmd: &str, context: &LintContext, cmd_context: &CommandContext) -> bool {
    let has_recursive = has_recursive_flag(cmd_context.args, context);

    for arg in cmd_context.args {
        let path_str = extract_arg_text(arg, context);
        if is_dangerous_path(path_str) || is_unvalidated_variable(path_str) {
            return true;
        }
    }

    // Even non-dangerous paths become dangerous with recursive flag
    has_recursive
}

/// Check if mv/cp commands have dangerous arguments
pub fn mv_cp_is_dangerous(_cmd: &str, context: &LintContext, cmd_context: &CommandContext) -> bool {
    for arg in cmd_context.args {
        let path_str = extract_arg_text(arg, context);
        if is_dangerous_path(path_str) || is_unvalidated_variable(path_str) {
            return true;
        }
    }

    false
}

/// Predicate that always returns true (for unconditional side effects)
pub const fn always(_cmd: &str, _context: &LintContext, _cmd_context: &CommandContext) -> bool {
    true
}

/// Check if print command is printing to stdout (not stderr)
pub fn prints_to_stdout(_cmd: &str, _context: &LintContext, cmd_context: &CommandContext) -> bool {
    !cmd_context.call.has_named_flag("stderr")
}

pub fn category_based_error(
    _cmd: &str,
    context: &LintContext,
    cmd_context: &CommandContext,
) -> bool {
    matches!(
        context
            .working_set
            .get_decl(cmd_context.call.decl_id)
            .signature()
            .category,
        nu_protocol::Category::Network | nu_protocol::Category::FileSystem
    )
}

/// Enhanced command registry with predicate-based side effects
static COMMAND_SIDE_EFFECTS: LazyLock<HashMap<&str, CommandPredicates>> = LazyLock::new(|| {
    [
        // File operations with context-dependent danger
        (
            "rm",
            CommandPredicates::new()
                .add(SideEffect::Error, always)
                .add(SideEffect::NoOutput, always)
                .add(SideEffect::Dangerous, rm_is_dangerous),
        ),
        (
            "mv",
            CommandPredicates::new()
                .add(SideEffect::Error, always)
                .add(SideEffect::NoOutput, always)
                .add(SideEffect::Dangerous, mv_cp_is_dangerous),
        ),
        (
            "cp",
            CommandPredicates::new()
                .add(SideEffect::Error, always)
                .add(SideEffect::Dangerous, mv_cp_is_dangerous),
        ),
        // Commands with category-based error detection
        (
            "open",
            CommandPredicates::new().add(SideEffect::Error, category_based_error),
        ),
        (
            "save",
            CommandPredicates::new().add(SideEffect::Error, category_based_error),
        ),
        (
            "from",
            CommandPredicates::new().add(SideEffect::Error, always),
        ),
        (
            "to",
            CommandPredicates::new().add(SideEffect::Error, always),
        ),
        // Network commands
        (
            "http get",
            CommandPredicates::new().add(SideEffect::Error, always),
        ),
        (
            "http post",
            CommandPredicates::new().add(SideEffect::Error, always),
        ),
        // Commands with no useful output
        (
            "mkdir",
            CommandPredicates::new()
                .add(SideEffect::Error, always)
                .add(SideEffect::NoOutput, always),
        ),
        (
            "touch",
            CommandPredicates::new()
                .add(SideEffect::Error, always)
                .add(SideEffect::NoOutput, always),
        ),
        (
            "cd",
            CommandPredicates::new()
                .add(SideEffect::Error, always)
                .add(SideEffect::NoOutput, always),
        ),
        (
            "sleep",
            CommandPredicates::new().add(SideEffect::NoOutput, always),
        ),
        (
            "use",
            CommandPredicates::new().add(SideEffect::NoOutput, always),
        ),
        (
            "hide",
            CommandPredicates::new().add(SideEffect::NoOutput, always),
        ),
        (
            "source",
            CommandPredicates::new()
                .add(SideEffect::Error, always)
                .add(SideEffect::NoOutput, always),
        ),
        (
            "source-env",
            CommandPredicates::new()
                .add(SideEffect::Error, always)
                .add(SideEffect::NoOutput, always),
        ),
        (
            "exit",
            CommandPredicates::new().add(SideEffect::NoOutput, always),
        ),
        (
            "error make",
            CommandPredicates::new()
                .add(SideEffect::Error, always)
                .add(SideEffect::NoOutput, always),
        ),
        (
            "input",
            CommandPredicates::new().add(SideEffect::Error, always),
        ),
        (
            "input list",
            CommandPredicates::new().add(SideEffect::Error, always),
        ),
        // Print operations
        (
            "print",
            CommandPredicates::new()
                .add(SideEffect::Print, prints_to_stdout)
                .add(SideEffect::NoOutput, always),
        ),
        // External commands that are unsafe in pipelines (need complete wrapper)
        (
            "curl",
            CommandPredicates::new().add(SideEffect::PipelineUnsafe, always),
        ),
        (
            "wget",
            CommandPredicates::new().add(SideEffect::PipelineUnsafe, always),
        ),
        (
            "find",
            CommandPredicates::new().add(SideEffect::PipelineUnsafe, always),
        ),
        (
            "grep",
            CommandPredicates::new().add(SideEffect::PipelineUnsafe, always),
        ),
        (
            "awk",
            CommandPredicates::new().add(SideEffect::PipelineUnsafe, always),
        ),
        (
            "sed",
            CommandPredicates::new().add(SideEffect::PipelineUnsafe, always),
        ),
        // External commands that are safe in pipelines (don't need complete wrapper)
        // These are simple, deterministic commands
        ("echo", CommandPredicates::new()), // No side effects - safe by absence
        ("printf", CommandPredicates::new()),
        ("true", CommandPredicates::new()),
        ("false", CommandPredicates::new()),
        ("yes", CommandPredicates::new()),
        ("date", CommandPredicates::new()),
        ("pwd", CommandPredicates::new()),
        ("whoami", CommandPredicates::new()),
        ("ls", CommandPredicates::new()),
        ("git", CommandPredicates::new()),
        ("uname", CommandPredicates::new()),
        ("arch", CommandPredicates::new()),
        ("hostname", CommandPredicates::new()),
        ("id", CommandPredicates::new()),
        ("uptime", CommandPredicates::new()),
        ("cal", CommandPredicates::new()),
        ("basename", CommandPredicates::new()),
        ("dirname", CommandPredicates::new()),
        ("realpath", CommandPredicates::new()),
        ("readlink", CommandPredicates::new()),
        ("env", CommandPredicates::new()),
        ("printenv", CommandPredicates::new()),
        ("tr", CommandPredicates::new()),
        ("cut", CommandPredicates::new()),
        ("paste", CommandPredicates::new()),
        ("column", CommandPredicates::new()),
        ("fmt", CommandPredicates::new()),
        ("fold", CommandPredicates::new()),
        ("expand", CommandPredicates::new()),
        ("unexpand", CommandPredicates::new()),
        ("bc", CommandPredicates::new()),
        ("dc", CommandPredicates::new()),
        ("expr", CommandPredicates::new()),
        ("mktemp", CommandPredicates::new()),
    ]
    .into_iter()
    .collect()
});

/// Main interface for checking if a command has a specific side effect
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

    let cmd_context = CommandContext {
        call,
        args: &external_args,
    };

    log::debug!(
        "Looking in registry for command '{command_name}' and side effect '{side_effect:?}'"
    );
    if let Some(predicates) = COMMAND_SIDE_EFFECTS.get(command_name) {
        for (effect, predicate) in &predicates.effects {
            if *effect == side_effect {
                log::debug!("Checking predicate for side effect '{side_effect:?}'");
                if predicate(command_name, context, &cmd_context) {
                    log::debug!("Predicate matched for side effect '{side_effect:?}'");
                    return true;
                }
                log::debug!("Predicate did not match for side effect '{side_effect:?}'");
            }
        }
    }
    log::debug!("No matching side effect '{side_effect:?}' found for command '{command_name}'");

    false
}

/// Check if an external command is safe to run without error handling
pub fn is_external_command_safe(command_name: &str) -> bool {
    // Command is safe if it's in registry but doesn't have PipelineUnsafe effect
    COMMAND_SIDE_EFFECTS
        .get(command_name)
        .is_some_and(|predicates| {
            !predicates
                .effects
                .iter()
                .any(|(effect, _)| *effect == SideEffect::PipelineUnsafe)
        })
}

/// Check if a command can produce errors (for try/do block recommendations)
pub fn can_error(command_name: &str, context: &LintContext, call: &Call) -> bool {
    has_side_effect(command_name, SideEffect::Error, context, call)
}

/// Check if an external command is known to produce no output (simple version
/// without Call context) This checks if the command is registered with
/// `NoOutput` side effect
pub fn external_command_has_no_output(command_name: &str) -> bool {
    COMMAND_SIDE_EFFECTS
        .get(command_name)
        .is_some_and(|predicates| {
            predicates
                .effects
                .iter()
                .any(|(effect, _)| *effect == SideEffect::NoOutput)
        })
}

/// Check if an external command is known to produce output
/// Commands in registry without `NoOutput` are assumed to produce output
/// Commands not in registry are conservatively assumed to produce output
pub fn external_command_has_output(command_name: &str) -> bool {
    // If not in registry at all, assume it produces output (conservative)
    // If in registry, check that it doesn't have NoOutput
    COMMAND_SIDE_EFFECTS
        .get(command_name)
        .is_none_or(|predicates| {
            !predicates
                .effects
                .iter()
                .any(|(effect, _)| *effect == SideEffect::NoOutput)
        })
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
