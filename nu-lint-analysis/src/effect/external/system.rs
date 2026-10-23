use nu_protocol::ast::ExternalArgument;

use super::{
    CommandEffects, ExternEffect,
    predicates::{always, get_subcommand, has_flag},
};
use crate::{context::LintContext, effect::CommonEffect};

fn systemctl_is_dangerous(context: &LintContext, args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args, context);
    matches!(
        subcommand,
        "stop" | "disable" | "mask" | "kill" | "reboot" | "poweroff" | "halt"
    )
}

fn kill_is_dangerous(context: &LintContext, args: &[ExternalArgument]) -> bool {
    has_flag(args, context, &["-9", "-KILL", "-SIGKILL"])
}

fn find_modifies_fs(context: &LintContext, args: &[ExternalArgument]) -> bool {
    has_flag(args, context, &["-delete", "-exec"])
}

fn find_is_dangerous(context: &LintContext, args: &[ExternalArgument]) -> bool {
    has_flag(args, context, &["-delete"])
}

pub const COMMANDS: &[CommandEffects] = &[
    // Process management
    (
        "kill",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                kill_is_dangerous,
            ),
        ],
    ),
    (
        "killall",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                always,
            ),
        ],
    ),
    (
        "pkill",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                always,
            ),
        ],
    ),
    (
        "pgrep",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    // System monitoring
    ("htop", &[(ExternEffect::NoDataInStdout, always)]),
    ("btop", &[(ExternEffect::NoDataInStdout, always)]),
    ("top", &[(ExternEffect::NoDataInStdout, always)]),
    (
        "free",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    // Service management
    (
        "systemctl",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                systemctl_is_dangerous,
            ),
        ],
    ),
    // Power management (extremely dangerous)
    (
        "reboot",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                always,
            ),
        ],
    ),
    (
        "shutdown",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                always,
            ),
        ],
    ),
    (
        "poweroff",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                always,
            ),
        ],
    ),
    (
        "halt",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                always,
            ),
        ],
    ),
    // User management
    (
        "sudo",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                always,
            ),
        ],
    ),
    (
        "su",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                always,
            ),
        ],
    ),
    (
        "doas",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                always,
            ),
        ],
    ),
    (
        "useradd",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                always,
            ),
        ],
    ),
    (
        "userdel",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                always,
            ),
        ],
    ),
    (
        "usermod",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                always,
            ),
        ],
    ),
    (
        "groupadd",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "groupdel",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "passwd",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                always,
            ),
        ],
    ),
    // Scheduling
    (
        "crontab",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "at",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    // Find (with special predicates)
    (
        "find",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, find_modifies_fs),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                find_is_dangerous,
            ),
        ],
    ),
    // Environment
    ("env", &[]),
    ("printenv", &[]),
    ("export", &[]),
    (
        "which",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "whereis",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "type",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    // Misc utilities
    ("echo", &[]),
    ("printf", &[]),
    ("date", &[]),
    ("cal", &[]),
    ("whoami", &[]),
    ("id", &[]),
    (
        "evtest",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    // Crypto
    (
        "openssl",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "gpg",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "ssh-keygen",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    // Cloud CLIs
    (
        "aws",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "gcloud",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "az",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
];
