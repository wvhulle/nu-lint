use nu_protocol::{
    BlockId,
    ast::{Block, Call, Expr, Expression, Pipeline},
};

use crate::{
    ast::{
        call::CallExt,
        effect::{SideEffect, has_side_effect},
        expression::ExpressionExt,
    },
    context::LintContext,
    rule::Rule,
    violation::Violation,
};

fn is_side_effect(expr: &Expression, context: &LintContext) -> bool {
    matches!(&expr.expr, Expr::ExternalCall(_, _))
        || matches!(&expr.expr, Expr::Call(call) if is_side_effect_call(call, context))
}

fn is_side_effect_call(call: &Call, context: &LintContext) -> bool {
    if call.is_control_flow_command(context) {
        return call.all_arg_expressions().iter().any(|arg| {
            arg.extract_block_id().is_some_and(|id| {
                has_side_effect_in_block(context.working_set.get_block(id), context)
            }) || is_side_effect(arg, context)
        });
    }

    let cmd_name = call.get_call_name(context);

    has_side_effect(&cmd_name, SideEffect::Print, context, call)
        || has_side_effect(&cmd_name, SideEffect::NoOutput, context, call)
        || has_side_effect(&cmd_name, SideEffect::Error, context, call)
        || matches!(
            context
                .working_set
                .get_decl(call.decl_id)
                .signature()
                .category,
            nu_protocol::Category::FileSystem
                | nu_protocol::Category::Network
                | nu_protocol::Category::System
        )
}

fn has_side_effect_in_block(block: &Block, context: &LintContext) -> bool {
    block.pipelines.iter().any(|pipeline| {
        pipeline
            .elements
            .iter()
            .any(|elem| is_side_effect(&elem.expr, context))
    })
}

fn count_leading_pure_pipelines(pipelines: &[Pipeline], context: &LintContext) -> usize {
    pipelines
        .iter()
        .take_while(|pipeline| {
            pipeline
                .elements
                .iter()
                .all(|elem| !is_side_effect(&elem.expr, context))
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
        pipeline
            .elements
            .iter()
            .any(|elem| is_side_effect(&elem.expr, context))
    })
}

fn analyze_function_body(
    block_id: BlockId,
    function_name: &str,
    context: &LintContext,
) -> Option<Violation> {
    log::debug!("Analyzing function '{function_name}' for pure statements before side effects");

    let block = context.working_set.get_block(block_id);
    log::debug!(
        "Function '{}' has {} pipelines",
        function_name,
        block.pipelines.len()
    );

    if block.pipelines.len() < 3 {
        log::debug!(
            "Function '{}' has too few pipelines ({} < 3), skipping",
            function_name,
            block.pipelines.len()
        );
        return None;
    }

    let pure_count = count_leading_pure_pipelines(&block.pipelines, context);
    log::debug!("Function '{function_name}' has {pure_count} consecutive pure statements");

    if pure_count < 2 {
        log::debug!(
            "Function '{function_name}' has too few pure statements ({pure_count} < 2), skipping"
        );
        return None;
    }

    if !has_side_effects_after(&block.pipelines, pure_count, context) {
        log::debug!(
            "Function '{function_name}' has no side effects after pure statements, skipping"
        );
        return None;
    }

    log::debug!("Function '{function_name}' violates pure_before_side_effects rule!");

    Some(
        Violation::new(
            "pure_before_side_effects",
            format!(
                "Function `{function_name}` has {pure_count} pure computation statement(s) before \
                 side effects"
            ),
            context.find_declaration_span(function_name),
        )
        .with_help(
            "Consider extracting the pure computation into a separate helper function. This \
             improves testability, reusability, and makes side effects more explicit.",
        ),
    )
}

fn check(context: &LintContext) -> Vec<Violation> {
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

pub const fn rule() -> Rule {
    Rule::new(
        "pure_before_side_effects",
        "Detect functions that have pure computation before side effects",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
