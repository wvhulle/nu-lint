use nu_protocol::ast::Call;

use crate::{ast::call::CallExt, context::LintContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IoType {
    FileSystem,
    Network,
    Print,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandEffect {
    /// Command only has side effects, returns nothing useful
    SideEffectOnly,
    /// Command has side effects but may also return data
    SideEffectWithData,
    /// Pure command with no side effects
    Pure,
}

const SIDE_EFFECT_ONLY_COMMANDS: &[&str] = &[
    "mkdir",
    "rm",
    "mv",
    "cp",
    "touch",
    "cd",
    "sleep",
    "hide",
    "use",
    "source",
    "source-env",
    "let",
    "mut",
    "const",
];

const SIDE_EFFECT_WITH_DATA_COMMANDS: &[&str] =
    &["print", "save", "exit", "error make", "input", "input list"];

const FILE_COMMANDS: &[&str] = &["save", "rm", "mv", "cp", "mkdir", "touch", "open"];
const PRINT_COMMANDS: &[&str] = &["print"];

pub fn classify_command_effect(command_name: &str) -> CommandEffect {
    if SIDE_EFFECT_ONLY_COMMANDS.contains(&command_name) {
        CommandEffect::SideEffectOnly
    } else if SIDE_EFFECT_WITH_DATA_COMMANDS.contains(&command_name) {
        CommandEffect::SideEffectWithData
    } else {
        CommandEffect::Pure
    }
}

pub fn has_side_effects(command_name: &str) -> bool {
    !matches!(classify_command_effect(command_name), CommandEffect::Pure)
}

pub fn is_side_effect_only(command_name: &str) -> bool {
    matches!(
        classify_command_effect(command_name),
        CommandEffect::SideEffectOnly
    )
}

pub fn get_io_type(command_name: &str, context: &LintContext, call: &Call) -> Option<IoType> {
    if PRINT_COMMANDS.contains(&command_name) {
        if call.has_named_flag("stderr") {
            return None;
        }
        return Some(IoType::Print);
    }

    if FILE_COMMANDS.contains(&command_name) {
        return Some(IoType::FileSystem);
    }

    if command_name.starts_with("http ") {
        return Some(IoType::Network);
    }

    let category = context
        .working_set
        .get_decl(call.decl_id)
        .signature()
        .category;

    match category {
        nu_protocol::Category::FileSystem => Some(IoType::FileSystem),
        nu_protocol::Category::Network => Some(IoType::Network),
        _ => None,
    }
}
