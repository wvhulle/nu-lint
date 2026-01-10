use nu_protocol::ast::ExternalArgument;

use crate::{
    context::LintContext,
    effect::{
        CommonEffect, is_dangerous_path, is_unvalidated_variable, matches_long_flag,
        matches_short_flag,
    },
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
    /// Produces useful output on `StdErr` (maybe in addition to `StdOut`)
    WritesDataToStdErr,
    /// This command performs network I/O operations
    ModifiesNetworkState,
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

pub fn extract_external_arg_text<'a>(arg: &ExternalArgument, context: &'a LintContext) -> &'a str {
    match arg {
        ExternalArgument::Regular(expr) | ExternalArgument::Spread(expr) => {
            context.plain_text(expr.span)
        }
    }
}

pub type ExternalSideEffectPredicate = fn(&LintContext<'_>, &[ExternalArgument]) -> bool;

const fn always(_context: &LintContext, _args: &[ExternalArgument]) -> bool {
    true
}

fn has_flag(args: &[ExternalArgument], context: &LintContext, patterns: &[&str]) -> bool {
    let matches_pattern = |arg_text: &str, pattern: &str| match pattern.strip_prefix("--") {
        Some(_) => matches_long_flag(arg_text, pattern),
        None => pattern
            .strip_prefix('-')
            .filter(|rest| rest.len() == 1)
            .and_then(|rest| rest.chars().next())
            .is_some_and(|flag_char| {
                matches_long_flag(arg_text, pattern) || matches_short_flag(arg_text, flag_char)
            }),
    };

    args.iter()
        .map(|arg| extract_external_arg_text(arg, context))
        .any(|arg_text| {
            patterns
                .iter()
                .any(|pattern| matches_pattern(arg_text, pattern))
        })
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

fn curl_modifies_fs(context: &LintContext, args: &[ExternalArgument]) -> bool {
    has_flag(args, context, &["-o", "--output", "-O", "--remote-name"])
}

fn tar_modifies_fs(context: &LintContext, args: &[ExternalArgument]) -> bool {
    has_flag(
        args,
        context,
        &["-x", "--extract", "--get", "-c", "--create"],
    )
}

fn sed_has_inplace(context: &LintContext, args: &[ExternalArgument]) -> bool {
    has_flag(args, context, &["-i", "--in-place"])
}

fn git_likely_errors(context: &LintContext, args: &[ExternalArgument]) -> bool {
    let subcommand = args
        .first()
        .map_or("", |arg| extract_external_arg_text(arg, context));

    matches!(
        subcommand,
        "clone"
            | "pull"
            | "push"
            | "fetch"
            | "checkout"
            | "switch"
            | "merge"
            | "rebase"
            | "cherry-pick"
            | "apply"
            | "commit"
            | "add"
            | "rm"
            | "mv"
            | "stash"
            | "restore"
            | "revert"
            | "remote"
            | "submodule"
            | "bisect"
            | "filter-branch"
            | "filter-repo"
    )
}

fn git_is_dangerous(context: &LintContext, args: &[ExternalArgument]) -> bool {
    let has_force = has_flag(args, context, &["-f", "--force"]);
    let has_hard = has_flag(args, context, &["--hard"]);
    let has_force_delete = has_flag(args, context, &["-D"]);
    let has_force_with_lease = has_flag(args, context, &["--force-with-lease"]);

    let subcommand = args
        .first()
        .map_or("", |arg| extract_external_arg_text(arg, context));
    log::debug!("Git called with subcommand {subcommand}");
    match subcommand {
        "push" => has_force || has_force_with_lease,
        "reset" => has_hard,
        "clean" => has_flag(args, context, &["-f", "-d", "-x"]),
        "branch" => has_force_delete,
        "filter-branch" | "filter-repo" => true,
        _ => false,
    }
}

fn git_modifies_filesystem(context: &LintContext, args: &[ExternalArgument]) -> bool {
    let subcommand = args
        .first()
        .map_or("", |arg| extract_external_arg_text(arg, context));

    match subcommand {
        "clone" | "pull" | "checkout" | "switch" | "reset" | "clean" | "merge" | "rebase"
        | "cherry-pick" | "apply" | "stash" | "restore" | "revert" | "commit" | "add" | "rm"
        | "mv" => true,
        "config" => {
            let second_arg = args
                .get(1)
                .map_or("", |arg| extract_external_arg_text(arg, context));
            !matches!(
                second_arg,
                "get" | "list" | "--list" | "-l" | "--get" | "--get-all" | "--get-regexp"
            ) && !has_flag(args, context, &["--list", "-l", "--get", "--get-all"])
        }
        _ => false,
    }
}

pub const EXTERNAL_COMMAND_SIDE_EFFECTS: &[(
    &str,
    &[(ExternEffect, ExternalSideEffectPredicate)],
)] = &[
    // Nushell std lib assert commands (parsed as external when std not loaded)
    ("assert", &[(ExternEffect::NoDataInStdout, always)]),
    (
        "rm",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::NoDataInStdout, always),
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
                always,
            ),
            (ExternEffect::NoDataInStdout, always),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                external_mv_cp_is_dangerous,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "cp",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::NoDataInStdout, always),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                external_mv_cp_is_dangerous,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "tar",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, tar_modifies_fs),
        ],
    ),
    ("echo", &[]),
    (
        "zip",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "unzip",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "rsync",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "scp",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "ssh",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "curl",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, curl_modifies_fs),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "wget",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "find",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "git",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                git_likely_errors,
            ),
            (ExternEffect::ModifiesFileSystem, git_modifies_filesystem),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                git_is_dangerous,
            ),
        ],
    ),
    (
        "grep",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "awk",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "sed",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, sed_has_inplace),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                sed_has_inplace,
            ),
        ],
    ),
    (
        "cat",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "head",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "tail",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "sort",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "uniq",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "wc",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "cut",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "xargs",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "ffmpeg",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::WritesDataToStdErr, always),
        ],
    ),
    (
        "evtest",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    // Modern CLI alternatives
    (
        "rg",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "fd",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    ("bat", &[]),
    ("exa", &[]),
    ("eza", &[]),
    ("htop", &[(ExternEffect::NoDataInStdout, always)]),
    ("btop", &[(ExternEffect::NoDataInStdout, always)]),
    (
        "duf",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "dust",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    // Package managers - Node.js ecosystem
    (
        "npm",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "npx",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "yarn",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "pnpm",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "node",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    // Package managers - Python ecosystem
    (
        "pip",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "pip3",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "pipx",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "uv",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "poetry",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "python",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "python3",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "pytest",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "mypy",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "ruff",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    // Package managers - Rust ecosystem
    (
        "cargo",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "rustc",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "rustup",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    // Container and orchestration tools
    (
        "docker",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "podman",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "kubectl",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "vagrant",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    // Build tools
    (
        "make",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "cmake",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "ninja",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "meson",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    // File system operations
    (
        "chmod",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::NoDataInStdout, always),
        ],
    ),
    (
        "chown",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::NoDataInStdout, always),
        ],
    ),
    (
        "ln",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::NoDataInStdout, always),
        ],
    ),
    (
        "touch",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::NoDataInStdout, always),
        ],
    ),
    (
        "mkdir",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::NoDataInStdout, always),
        ],
    ),
    (
        "rmdir",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::NoDataInStdout, always),
        ],
    ),
    // Data processing tools
    (
        "jq",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "yq",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    ("less", &[(ExternEffect::NoDataInStdout, always)]),
    // Additional common tools
    (
        "diff",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "patch",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "which",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    ("env", &[]),
    ("printenv", &[]),
    (
        "tee",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "dd",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
];
