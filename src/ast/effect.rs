use nu_protocol::ast::{Block, Call, Expr, Expression};

use crate::{
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IoType {
    FileSystem,
    Network,
    Print,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatementType {
    Pure,
    SideEffect,
    Control,
}

// Commands that have external side effects (file system, network, system state
// changes)
const EXTERNAL_SIDE_EFFECT_COMMANDS: &[&str] = &[
    "print",
    "save",
    "rm",
    "mv",
    "cp",
    "mkdir",
    "touch",
    "cd",
    "exit",
    "error make",
    "input",
    "input list",
    "sleep",
    "hide",
    "use",
    "source",
    "source-env",
];

// Commands that don't return useful data for pipelines (includes variable
// declarations)
const NO_DATA_RETURN_COMMANDS: &[&str] = &[
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

const FILE_COMMANDS: &[&str] = &["save", "rm", "mv", "cp", "mkdir", "touch", "open"];
const PRINT_COMMANDS: &[&str] = &["print"];

/// Check if a command has external side effects (used by
/// `pure_before_side_effects` rule)
pub fn has_side_effects(command_name: &str) -> bool {
    EXTERNAL_SIDE_EFFECT_COMMANDS.contains(&command_name)
}

/// Check if a command doesn't return useful data (used by
/// `print_and_return_data` rule)
pub fn is_side_effect_only(command_name: &str) -> bool {
    NO_DATA_RETURN_COMMANDS.contains(&command_name)
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

pub fn classify_expression(expr: &Expression, context: &LintContext) -> StatementType {
    match &expr.expr {
        Expr::ExternalCall(_, _) => StatementType::SideEffect,
        Expr::Call(call) => classify_call(call, context),
        Expr::Nothing => StatementType::Control,
        _ => StatementType::Pure,
    }
}

pub fn classify_call(call: &Call, context: &LintContext) -> StatementType {
    if call.is_control_flow_command(context) {
        return if has_side_effects_in_call(call, context) {
            StatementType::SideEffect
        } else {
            StatementType::Control
        };
    }

    let command_name = call.get_call_name(context);
    if has_side_effects(&command_name) {
        return StatementType::SideEffect;
    }

    let category = context
        .working_set
        .get_decl(call.decl_id)
        .signature()
        .category;
    match category {
        nu_protocol::Category::FileSystem
        | nu_protocol::Category::Network
        | nu_protocol::Category::System => StatementType::SideEffect,
        _ => StatementType::Pure,
    }
}

pub fn has_side_effects_in_call(call: &Call, context: &LintContext) -> bool {
    call.all_arg_expressions().iter().any(|arg_expr| {
        if let Some(block_id) = arg_expr.extract_block_id() {
            let block = context.working_set.get_block(block_id);
            return has_side_effects_in_block(block, context);
        }
        matches!(
            classify_expression(arg_expr, context),
            StatementType::SideEffect
        )
    })
}

pub fn has_side_effects_in_block(block: &Block, context: &LintContext) -> bool {
    block.pipelines.iter().any(|pipeline| {
        pipeline.elements.iter().any(|elem| {
            matches!(
                classify_expression(&elem.expr, context),
                StatementType::SideEffect
            )
        })
    })
}
