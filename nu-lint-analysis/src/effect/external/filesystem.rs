use super::{
    CommandEffects, ExternEffect,
    predicates::{always, has_dangerous_path_arg, rm_is_dangerous},
};
use crate::effect::CommonEffect;

pub const COMMANDS: &[CommandEffects] = &[
    (
        "rm",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::NoDataInStdout, always),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                rm_is_dangerous,
            ),
        ],
    ),
    (
        "mv",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::NoDataInStdout, always),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                has_dangerous_path_arg,
            ),
        ],
    ),
    (
        "cp",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::NoDataInStdout, always),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                has_dangerous_path_arg,
            ),
        ],
    ),
    (
        "chmod",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
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
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
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
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::NoDataInStdout, always),
        ],
    ),
    (
        "touch",
        &[
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::NoDataInStdout, always),
        ],
    ),
    (
        "mkdir",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
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
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::NoDataInStdout, always),
        ],
    ),
    (
        "truncate",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::NoDataInStdout, always),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                always,
            ),
        ],
    ),
    (
        "shred",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::NoDataInStdout, always),
            (
                ExternEffect::CommonEffect(CommonEffect::MayCauseDataLoss),
                always,
            ),
        ],
    ),
    // Read-only filesystem commands
    (
        "ls",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "stat",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "file",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "readlink",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "realpath",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    // Disk utilities
    (
        "df",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "du",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "mount",
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
        "umount",
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
        "fdisk",
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
        "parted",
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
        "mkfs",
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
        "dd",
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
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    // Modern alternatives
    (
        "exa",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "eza",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "duf",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "dust",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "ncdu",
        &[
            (ExternEffect::NoDataInStdout, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
];
