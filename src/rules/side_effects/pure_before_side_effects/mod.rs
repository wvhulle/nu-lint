use nu_protocol::{
    BlockId, Category,
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
        Category::FileSystem | Category::Network | Category::System => StatementType::SideEffect,
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

fn count_consecutive_pure_statements(pipelines: &[Pipeline], context: &LintContext) -> usize {
    pipelines
        .iter()
        .take_while(|pipeline| {
            pipeline.elements.iter().all(|elem| {
                matches!(
                    classify_expression(&elem.expr, context),
                    StatementType::Pure | StatementType::Control
                )
            })
        })
        .filter(|pipeline| {
            pipeline
                .elements
                .iter()
                .any(|elem| !matches!(&elem.expr.expr, Expr::Nothing))
        })
        .count()
}

fn has_side_effects_after(
    pipelines: &[Pipeline],
    pure_count: usize,
    context: &LintContext,
) -> bool {
    pipelines.iter().skip(pure_count).any(|pipeline| {
        pipeline.elements.iter().any(|elem| {
            matches!(
                classify_expression(&elem.expr, context),
                StatementType::SideEffect
            )
        })
    })
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

    if !has_side_effects_after(&block.pipelines, pure_count, context) {
        return None;
    }

    Some(
        RuleViolation::new_dynamic(
            "pure_before_side_effects",
            format!(
                "Function `{function_name}` has {pure_count} pure computation statement(s) before \
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
        .filter(|(_, name)| !context.is_exported_function(name))
        .filter_map(|(block_id, name)| analyze_function_body(*block_id, name, context))
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "pure_before_side_effects",
        RuleCategory::SideEffects,
        Severity::Warning,
        "Detect functions that have pure computation before side effects",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
