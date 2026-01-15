use nu_protocol::ast::{Expr, Pipeline, Traverse};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};
fn is_non_comment_statement(pipeline: &Pipeline) -> bool {
    pipeline
        .elements
        .iter()
        .any(|elem| !matches!(&elem.expr.expr, Expr::Nothing))
}
fn is_single_line_in_source(block_span: nu_protocol::Span, context: &LintContext) -> bool {
    let source_text = context.plain_text(block_span);
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

struct InlineSingleUseFunction;

impl DetectFix for InlineSingleUseFunction {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "inline_single_use_function"
    }

    fn short_description(&self) -> &'static str {
        "Detect single-line custom commands used only once that could be inlined"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/custom_commands.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let function_definitions = context.custom_commands();
        let has_main = function_definitions
            .iter()
            .any(super::super::ast::declaration::CustomCommandDef::is_main);
        if !has_main {
            return vec![];
        }
        let violations = function_definitions
            .iter()
            .filter(|def| !def.is_main())
            .filter(|def| !def.is_exported())
            .filter(|def| has_single_statement_body(def.body, context))
            .filter(|def| count_function_calls(&def.name, context) == 1)
            .map(|def| {
                let name_span = def.declaration_span(context);
                let block = context.working_set.get_block(def.body);
                // body_span is global (AST), name_span is file-relative - use AST span or
                // convert
                let body_span = block.span.unwrap_or_else(|| name_span.into());
                Detection::from_file_span(
                    format!(
                        "Function `{}` has a single-line body and is only used once",
                        def.name
                    ),
                    name_span,
                )
                .with_primary_label("single-use function")
                .with_extra_label("could be inlined", body_span)
            })
            .collect();
        Self::no_fix(violations)
    }
}

pub static RULE: &dyn Rule = &InlineSingleUseFunction;
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
