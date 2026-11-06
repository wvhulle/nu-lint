use nu_protocol::ast::ExternalArgument;

use super::{
    CommandEffects, ExternEffect,
    predicates::{always, get_subcommand, has_flag, literal_argument},
};
use crate::effect::CommonEffect;

fn docker_has_streaming_output(args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args);
    matches!(
        subcommand,
        Some("build" | "pull" | "push" | "run" | "logs" | "exec" | "compose" | "load" | "save")
    )
}

fn docker_is_dangerous(args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args);
    match subcommand {
        Some("rm" | "rmi") => has_flag(args, &["-f", "--force"]),
        Some("system") => {
            let second_arg = args.get(1).and_then(literal_argument);
            matches!(second_arg, Some("prune"))
        }
        Some("volume" | "network" | "image" | "container") => {
            let second_arg = args.get(1).and_then(literal_argument);
            matches!(second_arg, Some("rm" | "prune"))
        }
        _ => false,
    }
}

fn docker_modifies_fs(args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args);
    matches!(
        subcommand,
        Some("build" | "save" | "load" | "cp" | "export" | "import")
    )
}

fn kubectl_is_dangerous(args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args);
    matches!(
        subcommand,
        Some("delete" | "drain" | "cordon" | "taint" | "replace")
    )
}

fn kubectl_modifies_state(args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args);
    matches!(
        subcommand,
        Some(
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
    )
}

pub const COMMANDS: &[CommandEffects] = &[
    (
        "docker",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, docker_modifies_fs),
            (ExternEffect::ModifiesNetworkState, always),
            (
                ExternEffect::SlowStreamingOutput,
                docker_has_streaming_output,
            ),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                docker_is_dangerous,
            ),
        ],
    ),
    (
        "podman",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, docker_modifies_fs),
            (ExternEffect::ModifiesNetworkState, always),
            (
                ExternEffect::SlowStreamingOutput,
                docker_has_streaming_output,
            ),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                docker_is_dangerous,
            ),
        ],
    ),
    (
        "docker-compose",
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
    (
        "kubectl",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, kubectl_modifies_state),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                kubectl_is_dangerous,
            ),
        ],
    ),
    (
        "helm",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
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
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
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
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
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
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
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
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                always,
            ),
        ],
    ),
    (
        "tofu",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                always,
            ),
        ],
    ),
    (
        "pulumi",
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
    (
        "ansible",
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
    (
        "ansible-playbook",
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
