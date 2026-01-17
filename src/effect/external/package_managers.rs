use nu_protocol::ast::ExternalArgument;

use super::{
    CommandEffects, ExternEffect,
    predicates::{always, get_subcommand},
};
use crate::{context::LintContext, effect::CommonEffect};

fn npm_has_streaming_output(context: &LintContext, args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args, context);
    matches!(
        subcommand,
        "install" | "ci" | "run" | "test" | "build" | "start" | "publish" | "exec"
    )
}

fn pip_has_streaming_output(context: &LintContext, args: &[ExternalArgument]) -> bool {
    let subcommand = get_subcommand(args, context);
    matches!(subcommand, "install" | "download" | "wheel")
}

pub const COMMANDS: &[CommandEffects] = &[
    // Node.js ecosystem
    (
        "npm",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, npm_has_streaming_output),
        ],
    ),
    (
        "npx",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            // (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "yarn",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, npm_has_streaming_output),
        ],
    ),
    (
        "pnpm",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, npm_has_streaming_output),
        ],
    ),
    (
        "bun",
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
        "deno",
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
        "node",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    // Python ecosystem
    (
        "pip",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            // (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, pip_has_streaming_output),
        ],
    ),
    (
        "pip3",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            // (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, pip_has_streaming_output),
        ],
    ),
    (
        "pipx",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            // (ExternEffect::ModifiesFileSystem, always),
            // (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "uv",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            // (ExternEffect::ModifiesFileSystem, always),
            // (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "poetry",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            // (ExternEffect::ModifiesFileSystem, always),
            // (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "conda",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            // (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "python",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "python3",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "pytest",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "mypy",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "ruff",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "black",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "isort",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    // Ruby ecosystem
    (
        "ruby",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "gem",
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
        "bundle",
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
        "bundler",
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
        "rake",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    // PHP ecosystem
    (
        "php",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "composer",
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
    // Other languages
    (
        "perl",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "lua",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "luarocks",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    // Nix ecosystem
    (
        "nix",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            // (ExternEffect::ModifiesFileSystem, always),
            // (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "nix-build",
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
        "nix-shell",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            // (ExternEffect::ModifiesFileSystem, always),
            // (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "nix-env",
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
        "nixos-rebuild",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            // (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::SlowStreamingOutput, always),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "home-manager",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            // (ExternEffect::ModifiesFileSystem, always),
            // (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
];
