use nu_protocol::ast::ExternalArgument;

use super::{
    CommandEffects, ExternEffect,
    predicates::{always, get_subcommand},
};
use crate::{context::LintContext, effect::CommonEffect};

fn cargo_has_streaming_output(context: &LintContext, args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args, context);
    matches!(
        subcommand,
        "build"
            | "test"
            | "run"
            | "check"
            | "clippy"
            | "doc"
            | "install"
            | "publish"
            | "bench"
            | "fmt"
    )
}

fn cargo_modifies_fs(context: &LintContext, args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args, context);
    matches!(
        subcommand,
        "build"
            | "install"
            | "publish"
            | "new"
            | "init"
            | "add"
            | "remove"
            | "update"
            | "clean"
            | "doc"
            | "fmt"
    )
}

fn cargo_modifies_network(context: &LintContext, args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args, context);
    matches!(
        subcommand,
        "build" | "install" | "publish" | "update" | "fetch" | "search"
    )
}

pub const COMMANDS: &[CommandEffects] = &[
    // Rust
    (
        "cargo",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, cargo_modifies_fs),
            (ExternEffect::ModifiesNetworkState, cargo_modifies_network),
            (ExternEffect::StreamingOutput, cargo_has_streaming_output),
        ],
    ),
    (
        "rustc",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::StreamingOutput, always),
        ],
    ),
    (
        "rustup",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::StreamingOutput, always),
        ],
    ),
    (
        "rustfmt",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
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
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::StreamingOutput, always),
        ],
    ),
    (
        "cmake",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::StreamingOutput, always),
        ],
    ),
    (
        "ninja",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::StreamingOutput, always),
        ],
    ),
    (
        "meson",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::StreamingOutput, always),
        ],
    ),
    (
        "gcc",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "g++",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "clang",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "clang++",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
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
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::StreamingOutput, always),
        ],
    ),
    // Java/JVM
    (
        "javac",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "java",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "mvn",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::StreamingOutput, always),
        ],
    ),
    (
        "gradle",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::StreamingOutput, always),
        ],
    ),
    (
        "gradlew",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::StreamingOutput, always),
        ],
    ),
    // .NET
    (
        "dotnet",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::StreamingOutput, always),
        ],
    ),
    // Media processing
    (
        "ffmpeg",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::WritesDataToStdErr, always),
            (ExternEffect::StreamingOutput, always),
        ],
    ),
    (
        "ffprobe",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "convert",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "magick",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
];
