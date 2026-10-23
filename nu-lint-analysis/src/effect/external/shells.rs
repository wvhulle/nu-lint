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
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "sh",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "zsh",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "fish",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "dash",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "ksh",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "tcsh",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "csh",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    // Windows shells
    (
        "cmd",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "cmd.exe",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "powershell",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "powershell.exe",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "pwsh",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "pwsh.exe",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    // Script interpreters
    (
        "expect",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    // Database shells
    (
        "mysql",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "psql",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "sqlite3",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "mongo",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "mongosh",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "redis-cli",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
];
