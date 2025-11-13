use nu_protocol::{
    BlockId,
    ast::{Block, Call, Expr, Expression, Pipeline},
};

use crate::{
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

const SIDE_EFFECT_COMMANDS: &[&str] = &[
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
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatementType {
    Pure,
    SideEffect,
    Control,
}

fn is_side_effect_command(command_name: &str) -> bool {
    SIDE_EFFECT_COMMANDS.contains(&command_name)
}

fn classify_expression(expr: &Expression, context: &LintContext) -> StatementType {
    match &expr.expr {
        Expr::ExternalCall(_, _) => StatementType::SideEffect,
        Expr::Call(call) => classify_call(call, context),
        Expr::Nothing => StatementType::Control,
        _ => StatementType::Pure,
    }
}

fn classify_call(call: &Call, context: &LintContext) -> StatementType {
    if call.is_control_flow_command(context) {
        return if has_side_effects_in_call(call, context) {
            StatementType::SideEffect
        } else {
            StatementType::Control
        };
    }

    let command_name = call.get_call_name(context);
    if is_side_effect_command(&command_name) {
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

fn has_side_effects_in_call(call: &Call, context: &LintContext) -> bool {
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

fn has_side_effects_in_block(block: &Block, context: &LintContext) -> bool {
    block.pipelines.iter().any(|pipeline| {
        pipeline.elements.iter().any(|elem| {
            matches!(
                classify_expression(&elem.expr, context),
                StatementType::SideEffect
            )
        })
    })
}

fn is_pure_or_control(expr: &Expression, context: &LintContext) -> bool {
    matches!(
        classify_expression(expr, context),
        StatementType::Pure | StatementType::Control
    )
}

fn is_non_empty_pipeline(pipeline: &Pipeline) -> bool {
    pipeline
        .elements
        .iter()
        .any(|elem| !matches!(&elem.expr.expr, Expr::Nothing))
}

fn count_consecutive_pure_statements(pipelines: &[Pipeline], context: &LintContext) -> usize {
    pipelines
        .iter()
        .take_while(|pipeline| {
            pipeline
                .elements
                .iter()
                .all(|elem| is_pure_or_control(&elem.expr, context))
        })
        .filter(|pipeline| is_non_empty_pipeline(pipeline))
        .count()
}

fn has_side_effects_after_pure(pipelines: &[Pipeline], context: &LintContext) -> bool {
    let pure_count = count_consecutive_pure_statements(pipelines, context);

    pure_count > 0
        && pipelines.iter().skip(pure_count).any(|pipeline| {
            pipeline.elements.iter().any(|elem| {
                matches!(
                    classify_expression(&elem.expr, context),
                    StatementType::SideEffect
                )
            })
        })
}

fn is_exported_function(function_name: &str, context: &LintContext) -> bool {
    context
        .source
        .contains(&format!("export def {function_name}"))
}

fn analyze_function_body(
    block_id: BlockId,
    function_name: &str,
    context: &LintContext,
) -> Option<RuleViolation> {
    let block = context.working_set.get_block(block_id);

    if block.pipelines.len() < 3 {
        return None;
    }

    let pure_count = count_consecutive_pure_statements(&block.pipelines, context);

    if pure_count < 2 {
        return None;
    }

    if !has_side_effects_after_pure(&block.pipelines, context) {
        return None;
    }

    Some(
        RuleViolation::new_dynamic(
            "isolate_side_effects",
            format!(
                "Function `{function_name}` mixes {pure_count} pure computation statement(s) with \
                 side effects"
            ),
            context.find_declaration_span(function_name),
        )
        .with_suggestion_static(
            "Consider extracting the pure computation into a separate helper function. This \
             improves testability, reusability, and makes side effects more explicit.",
        ),
    )
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let function_definitions = context.collect_function_definitions();

    let has_main = function_definitions.values().any(|name| name == "main");
    if !has_main {
        return vec![];
    }

    function_definitions
        .iter()
        .filter(|(_, name)| *name != "main")
        .filter(|(_, name)| !is_exported_function(name, context))
        .filter_map(|(block_id, name)| analyze_function_body(*block_id, name, context))
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "isolate_side_effects",
        RuleCategory::CodeQuality,
        Severity::Info,
        "Detect functions that mix pure computation with side effects",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
