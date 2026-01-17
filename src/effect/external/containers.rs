use nu_protocol::ast::ExternalArgument;

use super::{
    CommandEffects, ExternEffect,
    predicates::{always, extract_external_arg_text, get_subcommand, has_flag},
};
use crate::{context::LintContext, effect::CommonEffect};

fn docker_has_streaming_output(context: &LintContext, args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args, context);
    matches!(
        subcommand,
        "build" | "pull" | "push" | "run" | "logs" | "exec" | "compose" | "load" | "save"
    )
}

fn docker_is_dangerous(context: &LintContext, args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args, context);
    match subcommand {
        "rm" | "rmi" => has_flag(args, context, &["-f", "--force"]),
        "system" => {
            let second_arg = args
                .get(1)
                .map_or("", |arg| extract_external_arg_text(arg, context));
            matches!(second_arg, "prune")
        }
        "volume" | "network" | "image" | "container" => {
            let second_arg = args
                .get(1)
                .map_or("", |arg| extract_external_arg_text(arg, context));
            matches!(second_arg, "rm" | "prune")
        }
        _ => false,
    }
}

fn docker_modifies_fs(context: &LintContext, args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args, context);
    matches!(
        subcommand,
        "build" | "save" | "load" | "cp" | "export" | "import"
    )
}

fn kubectl_is_dangerous(context: &LintContext, args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args, context);
    matches!(
        subcommand,
        "delete" | "drain" | "cordon" | "taint" | "replace"
    )
}

fn kubectl_modifies_state(context: &LintContext, args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args, context);
    matches!(
        subcommand,
        "apply"
            | "create"
            | "delete"
            | "patch"
            | "replace"
            | "scale"
            | "edit"
            | "rollout"
            | "drain"
            | "cordon"
            | "uncordon"
            | "taint"
            | "label"
            | "annotate"
            | "set"
    )
}

pub const COMMANDS: &[CommandEffects] = &[
    (
        "docker",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, docker_modifies_fs),
            (ExternEffect::ModifiesNetworkState, always),
            (
                ExternEffect::SlowStreamingOutput,
                docker_has_streaming_output,
            ),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                docker_is_dangerous,
            ),
        ],
    ),
    (
        "podman",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, docker_modifies_fs),
            (ExternEffect::ModifiesNetworkState, always),
            (
                ExternEffect::SlowStreamingOutput,
                docker_has_streaming_output,
            ),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                docker_is_dangerous,
            ),
        ],
    ),
    (
        "docker-compose",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "kubectl",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, kubectl_modifies_state),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                kubectl_is_dangerous,
            ),
        ],
    ),
    (
        "helm",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "minikube",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "kind",
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
        "vagrant",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    // Infrastructure as Code
    (
        "terraform",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "tofu",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "pulumi",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "ansible",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "ansible-playbook",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
];
