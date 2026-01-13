use super::{CommandEffects, ExternEffect, predicates::always};
use crate::effect::CommonEffect;

pub const COMMANDS: &[CommandEffects] = &[
    // Nushell std lib (parsed as external when std not loaded)
    ("assert", &[(ExternEffect::NoDataInStdout, always)]),
    ("assert equal", &[(ExternEffect::NoDataInStdout, always)]),
    ("assert not", &[(ExternEffect::NoDataInStdout, always)]),
    ("assert error", &[(ExternEffect::NoDataInStdout, always)]),
    // Unix shells
    (
        "bash",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "sh",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "zsh",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "fish",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "dash",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "ksh",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "tcsh",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "csh",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    // Windows shells
    (
        "cmd",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "cmd.exe",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "powershell",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "powershell.exe",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "pwsh",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "pwsh.exe",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    // Script interpreters
    (
        "expect",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    // Database shells
    (
        "mysql",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "psql",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "sqlite3",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "mongo",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "mongosh",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "redis-cli",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
];
