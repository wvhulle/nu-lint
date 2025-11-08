use nu_protocol::ast::{Expr, Pipeline, Traverse};

use crate::{
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

fn count_non_comment_statements(pipeline: &Pipeline) -> usize {
    pipeline
        .elements
        .iter()
        .filter(|elem| !matches!(&elem.expr.expr, Expr::Nothing))
        .count()
}

fn has_single_statement_body(block_id: nu_protocol::BlockId, context: &LintContext) -> bool {
    let block = context.working_set.get_block(block_id);

    // Count pipelines that have actual content (not just comments/nothing)
    let non_empty_pipelines: Vec<_> = block
        .pipelines
        .iter()
        .filter(|p| count_non_comment_statements(p) > 0)
        .collect();

    // Must have exactly one non-empty pipeline with exactly one element
    if !(non_empty_pipelines.len() == 1 && non_empty_pipelines[0].elements.len() == 1) {
        return false;
    }

    // Check if the function body is truly single-line in source
    if let Some(block_span) = block.span {
        let source_text = &context.source[block_span.start..block_span.end];
        let line_count = source_text.lines().count();
        // If body spans more than 3 lines (accounting for braces), consider it multi-line
        if line_count > 3 {
            return false;
        }
    }

    true
}

fn count_function_calls(function_name: &str, context: &LintContext) -> usize {
    let function_decl_id = context
        .working_set
        .find_decl(function_name.as_bytes())
        .expect("Function should exist");

    let mut all_calls = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            if let Expr::Call(call) = &expr.expr {
                vec![call.decl_id]
            } else {
                vec![]
            }
        },
        &mut all_calls,
    );

    all_calls
        .iter()
        .filter(|&&id| id == function_decl_id)
        .count()
}

fn is_exported_function(function_name: &str, context: &LintContext) -> bool {
    // Check if the source code contains "export def <function_name>"
    context
        .source
        .contains(&format!("export def {function_name}"))
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let function_definitions = context.collect_function_definitions();

    // Only check if there's a main function (similar to unused_helper_functions)
    if !function_definitions.values().any(|name| name == "main") {
        return vec![];
    }

    let mut violations = Vec::new();

    for (block_id, function_name) in &function_definitions {
        // Skip main function
        if function_name == "main" {
            continue;
        }

        // Skip exported functions
        if is_exported_function(function_name, context) {
            continue;
        }

        if !has_single_statement_body(*block_id, context) {
            continue;
        }

        let call_count = count_function_calls(function_name, context);

        if call_count == 1 {
            let span = context.find_declaration_span(function_name);
            violations.push(
                RuleViolation::new_dynamic(
                    "inline_single_use_function",
                    format!(
                        "Function `{function_name}` has a single-line body and is only used once"
                    ),
                    span,
                )
                .with_suggestion_static(
                    "Consider inlining this function at its call site. Single-line helper \
                     functions used only once may add unnecessary indirection and reduce code \
                     clarity.",
                ),
            );
        }
    }

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "inline_single_use_function",
        RuleCategory::CodeQuality,
        Severity::Info,
        "Detect single-line custom commands used only once that could be inlined",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
