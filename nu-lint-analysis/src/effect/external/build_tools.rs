use nu_protocol::ast::ExternalArgument;

use super::{
    CommandEffects, ExternEffect,
    predicates::{always, get_subcommand},
};
use crate::effect::CommonEffect;

fn cargo_has_streaming_output(args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args);
    matches!(
        subcommand,
        Some("build" | "test" | "run" | "install" | "bench")
    )
}

fn cargo_modifies_fs(args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args);
    matches!(
        subcommand,
        Some("build" | "install" | "new" | "init" | "add" | "remove" | "update" | "clean" | "fmt")
    )
}

fn cargo_modifies_network(args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args);
    matches!(subcommand, Some("publish"))
}

pub const COMMANDS: &[CommandEffects] = &[
    // Rust
    (
        "cargo",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, cargo_modifies_fs),
            (ExternEffect::ModifiesNetworkState, cargo_modifies_network),
            (
                ExternEffect::SlowStreamingOutput,
                cargo_has_streaming_output,
            ),
        ],
    ),
    (
        "rustc",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "rustup",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            // (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "rustfmt",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    // C/C++
    (
        "make",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "cmake",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "ninja",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "meson",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "gcc",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "g++",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "clang",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "clang++",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    // Go
    (
        "go",
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
    // Java/JVM
    (
        "javac",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "java",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "mvn",
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
        "gradle",
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
        "gradlew",
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
    // .NET
    (
        "dotnet",
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
    // Media processing
    (
        "ffmpeg",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::WritesDataToStdErr, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "ffprobe",
        &[(
            ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
            always,
        )],
    ),
    (
        "convert",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "magick",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
];
