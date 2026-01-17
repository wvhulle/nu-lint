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
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::NoDataInStdout, always),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                rm_is_dangerous,
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
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::NoDataInStdout, always),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                has_dangerous_path_arg,
            ),
        ],
    ),
    (
        "cp",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::NoDataInStdout, always),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                has_dangerous_path_arg,
            ),
        ],
    ),
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
    (
        "truncate",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::NoDataInStdout, always),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "shred",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::NoDataInStdout, always),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    // Read-only filesystem commands
    (
        "ls",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "stat",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "file",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "readlink",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "realpath",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    // Disk utilities
    (
        "df",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "du",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "mount",
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
        "umount",
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
        "fdisk",
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
        "parted",
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
        "mkfs",
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
        "dd",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    // Modern alternatives
    (
        "exa",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "eza",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
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
    (
        "ncdu",
        &[
            (ExternEffect::NoDataInStdout, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
];
