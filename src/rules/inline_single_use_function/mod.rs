use nu_protocol::ast::{Expr, Pipeline, Traverse};

use crate::{context::LintContext, rule::Rule, violation::Violation};
fn is_non_comment_statement(pipeline: &Pipeline) -> bool {
    pipeline
        .elements
        .iter()
        .any(|elem| !matches!(&elem.expr.expr, Expr::Nothing))
}
fn is_single_line_in_source(block_span: nu_protocol::Span, context: &LintContext) -> bool {
    let source_text =
        std::str::from_utf8(context.working_set.get_span_contents(block_span)).unwrap_or("");
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
    let Some(function_decl_id) = context.working_set.find_decl(function_name.as_bytes()) else {
        log::debug!("Function '{function_name}' not found in working set, skipping");
        return 0;
    };
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
    context.source_contains(&format!("export def {function_name}"))
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
        .map(|(block_id, function_name)| {
            let name_span = context.find_declaration_span(function_name);
            let block = context.working_set.get_block(*block_id);
            // body_span is global (AST), name_span is file-relative - use AST span or
            // convert
            let body_span = block.span.unwrap_or_else(|| name_span.into());
            Violation::with_file_span(
                format!("Function `{function_name}` has a single-line body and is only used once"),
                name_span,
            )
            .with_primary_label("single-use function")
            .with_extra_label("could be inlined", body_span)
            .with_help(
                "Consider inlining this function at its call site. Single-line helper functions \
                 used only once may add unnecessary indirection and reduce code clarity.",
            )
        })
        .collect()
}
pub const fn rule() -> Rule {
    Rule::new(
        "inline_single_use_function",
        "Detect single-line custom commands used only once that could be inlined",
        check,
    )
    .with_doc_url("https://www.nushell.sh/book/custom_commands.html")
}
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
