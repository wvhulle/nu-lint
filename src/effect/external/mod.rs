mod archives;
mod build_tools;
mod containers;
mod filesystem;
mod git;
mod network;
mod package_managers;
mod predicates;
mod shells;
mod system;
mod text_processing;

use nu_protocol::ast::ExternalArgument;
pub use predicates::extract_external_arg_text;

use super::CommonEffect;
use crate::context::LintContext;

/// Things that may happen at runtime for external commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternEffect {
    /// Effect that is common between built-in and external commands.
    CommonEffect(CommonEffect),
    /// Silent, does not produce useful output
    NoDataInStdout,
    /// This command modifies the file system
    ModifiesFileSystem,
    /// Produces useful output on `StdErr` (maybe in addition to `StdOut`)
    WritesDataToStdErr,
    /// This command performs network I/O operations
    ModifiesNetworkState,
    /// Output is useful to see in real-time (progress bars, build output, etc.)
    SlowStreamingOutput,
}

pub type CommandEffects = (
    &'static str,
    &'static [(ExternEffect, predicates::Predicate)],
);

/// All external command side effects, merged from domain-specific modules.
const ALL_COMMANDS: &[&[CommandEffects]] = &[
    filesystem::COMMANDS,
    archives::COMMANDS,
    text_processing::COMMANDS,
    network::COMMANDS,
    git::COMMANDS,
    build_tools::COMMANDS,
    containers::COMMANDS,
    package_managers::COMMANDS,
    system::COMMANDS,
    shells::COMMANDS,
];

pub fn has_external_side_effect(
    command_name: &str,
    side_effect: ExternEffect,
    context: &LintContext,
    args: &[ExternalArgument],
) -> bool {
    log::trace!("Checking external side effect '{side_effect:?}' for command '{command_name}'");

    for commands in ALL_COMMANDS {
        if let Some((_, effects)) = commands.iter().find(|(name, _)| *name == command_name)
            && let Some((_, predicate)) = effects.iter().find(|(effect, _)| *effect == side_effect)
        {
            log::trace!("Checking external predicate for side effect '{side_effect:?}'");
            let result = predicate(context, args);
            if result {
                log::trace!("External predicate matched for side effect '{side_effect:?}'");
            }
            return result;
        }
    }

    log::trace!(
        "No matching external side effect '{side_effect:?}' found for command '{command_name}'"
    );
    false
}

pub fn has_external_recursive_flag(args: &[ExternalArgument], context: &LintContext) -> bool {
    args.iter().any(|arg| {
        let arg_text = extract_external_arg_text(arg, context);
        matches!(
            arg_text,
            text if text.contains("-r")
                || text.contains("--recursive")
                || text.contains("-rf")
                || text.contains("-fr")
                || text.contains("--force")
        )
    })
}
