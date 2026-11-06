use nu_protocol::ast::ExternalArgument;

use super::{
    CommandEffects, ExternEffect,
    predicates::{always, get_subcommand, has_flag, literal_argument},
};
use crate::effect::CommonEffect;

fn git_likely_errors(args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args);
    matches!(
        subcommand,
        Some(
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
                | "restore"
                | "revert"
                | "remote"
                | "submodule"
                | "bisect"
                | "filter-branch"
                | "filter-repo"
                | "worktree"
        )
    )
}

fn git_is_dangerous(args: &[ExternalArgument]) -> bool {
    let has_force = has_flag(args, &["-f", "--force"]);
    let has_hard = has_flag(args, &["--hard"]);
    let has_force_delete = has_flag(args, &["-D"]);
    let has_force_with_lease = has_flag(args, &["--force-with-lease"]);

    let subcommand = get_subcommand(args);
    match subcommand {
        Some("push") => has_force || has_force_with_lease,
        Some("reset") => has_hard,
        Some("clean") => has_flag(args, &["-f", "-d", "-x"]),
        Some("branch") => has_force_delete,
        Some("filter-branch" | "filter-repo" | "gc") => true,
        Some("reflog") => {
            let second_arg = args.get(1).and_then(literal_argument);
            matches!(second_arg, Some("delete" | "expire"))
        }
        _ => false,
    }
}

fn git_modifies_filesystem(args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args);
    match subcommand {
        Some(
            "clone" | "pull" | "checkout" | "switch" | "reset" | "clean" | "merge" | "rebase"
            | "cherry-pick" | "apply" | "stash" | "restore" | "revert" | "commit" | "rm" | "mv"
            | "worktree" | "submodule",
        ) => true,
        Some("config") => {
            let second_arg = args.get(1).and_then(literal_argument);
            !matches!(
                second_arg,
                Some("get" | "list" | "--list" | "-l" | "--get" | "--get-all" | "--get-regexp")
            ) && !has_flag(args, &["--list", "-l", "--get", "--get-all"])
        }
        _ => false,
    }
}

fn git_has_streaming_output(args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args);
    matches!(subcommand, Some("clone" | "pull" | "push" | "fetch"))
}

fn git_modifies_network(args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args);
    matches!(subcommand, Some("push"))
}

pub const COMMANDS: &[CommandEffects] = &[
    (
        "git",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                git_likely_errors,
            ),
            (ExternEffect::ModifiesFileSystem, git_modifies_filesystem),
            (ExternEffect::ModifiesNetworkState, git_modifies_network),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                git_is_dangerous,
            ),
            (ExternEffect::SlowStreamingOutput, git_has_streaming_output),
        ],
    ),
    // Git-related tools
    (
        "gh",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "hub",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "git-lfs",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
];
