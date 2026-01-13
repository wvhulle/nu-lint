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
        "stop"
            | "disable"
            | "mask"
            | "kill"
            | "reset-failed"
            | "daemon-reload"
            | "reboot"
            | "poweroff"
            | "halt"
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
        "ps",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "kill",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                kill_is_dangerous,
            ),
        ],
    ),
    (
        "killall",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "pkill",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "pgrep",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
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
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    ("uptime", &[]),
    ("uname", &[]),
    (
        "hostname",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    // Service management
    (
        "systemctl",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                systemctl_is_dangerous,
            ),
        ],
    ),
    (
        "service",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "journalctl",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "dmesg",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    // Power management (extremely dangerous)
    (
        "reboot",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "shutdown",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "poweroff",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "halt",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    // User management
    (
        "sudo",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "su",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "doas",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "useradd",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "userdel",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "usermod",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "groupadd",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "groupdel",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "passwd",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    // Scheduling
    (
        "crontab",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "at",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
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
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, find_modifies_fs),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
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
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "whereis",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "type",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
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
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    // Crypto
    (
        "openssl",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "gpg",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "ssh-keygen",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
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
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
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
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
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
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
];
