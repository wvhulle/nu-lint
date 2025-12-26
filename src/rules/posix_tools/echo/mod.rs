use nu_protocol::{
    Span,
    ast::{Block, Expr, Pipeline, PipelineElement},
};

use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

/// Semantic fix data: stores the span of the echo call
pub struct FixData {
    element_span: Span,
}

fn uses_echo(element: &PipelineElement, context: &LintContext) -> bool {
    match &element.expr.expr {
        Expr::Call(call) => call.is_call_to_command("echo", context),
        Expr::ExternalCall(head, _) => context.get_span_text(head.span) == "echo",
        _ => false,
    }
}

fn extract_echo_args(code_snippet: &str) -> &str {
    code_snippet
        .strip_prefix("^echo")
        .or_else(|| code_snippet.strip_prefix("echo"))
        .unwrap_or("")
        .trim()
}

fn extract_nested_block_ids(
    element: &PipelineElement,
    context: &LintContext,
) -> Vec<nu_protocol::BlockId> {
    use nu_protocol::ast::Traverse;

    let mut blocks = Vec::new();
    element.expr.flat_map(
        context.working_set,
        &|expr| match &expr.expr {
            Expr::Block(block_id) | Expr::Closure(block_id) | Expr::Subexpression(block_id) => {
                vec![*block_id]
            }
            _ => vec![],
        },
        &mut blocks,
    );
    blocks
}

fn detect_element(
    element: &PipelineElement,
    _idx: usize,
    _pipeline: &Pipeline,
    context: &LintContext,
) -> Vec<(Detection, FixData)> {
    let mut violations = Vec::new();

    if uses_echo(element, context) {
        let message = "Avoid 'echo' - it's just an identity function. Use the value directly, or \
                       'print' for debugging";
        let violation = Detection::from_global_span(message, element.expr.span);
        let fix_data = FixData {
            element_span: element.expr.span,
        };
        violations.push((violation, fix_data));
    }

    let nested_violations = extract_nested_block_ids(element, context)
        .iter()
        .flat_map(|&block_id| {
            let block = context.working_set.get_block(block_id);
            detect_block(block, context)
        })
        .collect::<Vec<_>>();

    violations.extend(nested_violations);
    violations
}

fn detect_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<(Detection, FixData)> {
    pipeline
        .elements
        .iter()
        .enumerate()
        .flat_map(|(idx, element)| detect_element(element, idx, pipeline, context))
        .collect()
}

fn detect_block(block: &Block, context: &LintContext) -> Vec<(Detection, FixData)> {
    block
        .pipelines
        .iter()
        .flat_map(|pipeline| detect_pipeline(pipeline, context))
        .collect()
}

struct UseBuiltinEcho;

impl DetectFix for UseBuiltinEcho {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "use_builtin_echo"
    }

    fn explanation(&self) -> &'static str {
        "D not use builtin 'echo' as it's just an identity function"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/thinking_in_nu.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let block: &Block = context.ast;
        block
            .pipelines
            .iter()
            .flat_map(|pipeline| detect_pipeline(pipeline, context))
            .collect()
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let code_snippet = context.get_span_text(fix_data.element_span);
        let args = extract_echo_args(code_snippet);

        if args.is_empty() {
            None
        } else {
            Some(Fix::with_explanation(
                format!("Replace '{code_snippet}' with '{args}'"),
                vec![Replacement::new(fix_data.element_span, args.to_string())],
            ))
        }
    }
}

pub static RULE: &dyn Rule = &UseBuiltinEcho;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
