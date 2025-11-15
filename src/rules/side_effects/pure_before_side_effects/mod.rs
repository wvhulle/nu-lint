use nu_protocol::{
    BlockId,
    ast::{Expr, Pipeline},
};

use crate::{
    LintLevel,
    ast::effect::{StatementType, classify_expression},
    context::LintContext,
    rule::Rule,
    violation::Violation,
};

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
) -> Option<Violation> {
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
        Violation::new_dynamic(
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

pub fn rule() -> Rule {
    Rule::new(
        "pure_before_side_effects",
        LintLevel::Allow,
        "Detect functions that have pure computation before side effects",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
