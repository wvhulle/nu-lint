use nu_protocol::ast::ExternalArgument;

use super::{
    CommandEffects, ExternEffect,
    predicates::{always, extract_external_arg_text, get_subcommand, has_flag},
};
use crate::{context::LintContext, effect::CommonEffect};

fn git_likely_errors(context: &LintContext, args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args, context);
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
            | "restore"
            | "revert"
            | "remote"
            | "submodule"
            | "bisect"
            | "filter-branch"
            | "filter-repo"
            | "worktree"
    )
}

fn git_is_dangerous(context: &LintContext, args: &[ExternalArgument]) -> bool {
    let has_force = has_flag(args, context, &["-f", "--force"]);
    let has_hard = has_flag(args, context, &["--hard"]);
    let has_force_delete = has_flag(args, context, &["-D"]);
    let has_force_with_lease = has_flag(args, context, &["--force-with-lease"]);

    let subcommand = get_subcommand(args, context);
    match subcommand {
        "push" => has_force || has_force_with_lease,
        "reset" => has_hard,
        "clean" => has_flag(args, context, &["-f", "-d", "-x"]),
        "branch" => has_force_delete,
        "filter-branch" | "filter-repo" | "gc" => true,
        "reflog" => {
            let second_arg = args
                .get(1)
                .map_or("", |arg| extract_external_arg_text(arg, context));
            matches!(second_arg, "delete" | "expire")
        }
        _ => false,
    }
}

fn git_modifies_filesystem(context: &LintContext, args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args, context);
    match subcommand {
        "clone" | "pull" | "checkout" | "switch" | "reset" | "clean" | "merge" | "rebase"
        | "cherry-pick" | "apply" | "stash" | "restore" | "revert" | "commit" | "rm" | "mv"
        | "worktree" | "submodule" => true,
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

fn git_has_streaming_output(context: &LintContext, args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args, context);
    matches!(subcommand, "clone" | "pull" | "push" | "fetch")
}

fn git_modifies_network(context: &LintContext, args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args, context);
    matches!(subcommand, "push")
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
