use nu_protocol::ast::{Expr, Pipeline, Traverse};

use crate::{context::LintContext, rule::Rule, violation::Violation};
fn is_non_comment_statement(pipeline: &Pipeline) -> bool {
    pipeline
        .elements
        .iter()
        .any(|elem| !matches!(&elem.expr.expr, Expr::Nothing))
}
fn is_single_line_in_source(block_span: nu_protocol::Span, context: &LintContext) -> bool {
    let source_text = &context.source[block_span.start..block_span.end];
    source_text.lines().count() <= 3
}
fn has_single_statement_body(block_id: nu_protocol::BlockId, context: &LintContext) -> bool {
    let block = context.working_set.get_block(block_id);
    let has_single_pipeline = block
        .pipelines
        .iter()
        .filter(|p| is_non_comment_statement(p))
        .count()
        == 1;
    let has_single_element = block
        .pipelines
        .iter()
        .find(|p| is_non_comment_statement(p))
        .is_some_and(|p| p.elements.len() == 1);
    has_single_pipeline
        && has_single_element
        && block
            .span
            .is_none_or(|span| is_single_line_in_source(span, context))
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
            matches!(&expr.expr, Expr::Call(call) if call.decl_id == function_decl_id)
                .then_some(function_decl_id)
                .into_iter()
                .collect()
        },
        &mut all_calls,
    );
    all_calls.len()
}
fn is_exported_function(function_name: &str, context: &LintContext) -> bool {
    context
        .source
        .contains(&format!("export def {function_name}"))
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
        .filter(|(_, name)| !is_exported_function(name, context))
        .filter(|(block_id, _)| has_single_statement_body(**block_id, context))
        .filter(|(_, name)| count_function_calls(name, context) == 1)
        .map(|(_, function_name)| {
            Violation::new_dynamic(
                "inline_single_use_function",
                format!("Function `{function_name}` has a single-line body and is only used once"),
                context.find_declaration_span(function_name),
            )
            .with_suggestion_static(
                "Consider inlining this function at its call site. Single-line helper functions \
                 used only once may add unnecessary indirection and reduce code clarity.",
            )
        })
        .collect()
}
pub fn rule() -> Rule {
    Rule::new(
        "inline_single_use_function",
        "Detect single-line custom commands used only once that could be inlined",
        check,
    )
}
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
