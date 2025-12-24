use nu_protocol::ast::{Block, Expr, Pipeline, PipelineElement};

use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement, Violation},
};

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

fn generate_fix(code_snippet: &str, span: nu_protocol::Span) -> Option<Fix> {
    let args = extract_echo_args(code_snippet);

    if args.is_empty() {
        None
    } else {
        Some(Fix::with_explanation(
            format!("Replace '{code_snippet}' with '{args}'"),
            vec![Replacement::new(span, args.to_string())],
        ))
    }
}

fn get_pipeline_continuation<'a>(
    pipeline: &Pipeline,
    element_idx: usize,
    context: &'a LintContext,
) -> Option<&'a str> {
    pipeline.elements.get(element_idx + 1).map(|next_element| {
        let start = next_element.expr.span.start;
        let end = pipeline.elements.last().unwrap().expr.span.end;
        context.get_span_text(nu_protocol::Span::new(start, end))
    })
}

fn create_violation(
    element: &PipelineElement,
    _pipeline_continuation: Option<&str>,
    context: &LintContext,
) -> Violation {
    let message = "Avoid 'echo' - it's just an identity function. Use the value directly, or \
                   'print' for debugging";
    let code_snippet = context.get_span_text(element.expr.span);
    let fix = generate_fix(code_snippet, element.expr.span);

    let violation = Violation::new(message, element.expr.span);

    match fix {
        Some(f) => violation.with_fix(f),
        None => violation,
    }
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

fn check_element(
    element: &PipelineElement,
    idx: usize,
    pipeline: &Pipeline,
    context: &LintContext,
) -> Vec<Violation> {
    let mut violations = Vec::new();

    if uses_echo(element, context) {
        let pipeline_continuation = get_pipeline_continuation(pipeline, idx, context);
        violations.push(create_violation(element, pipeline_continuation, context));
    }

    let nested_violations = extract_nested_block_ids(element, context)
        .iter()
        .flat_map(|&block_id| {
            let block = context.working_set.get_block(block_id);
            check_block(block, context)
        })
        .collect::<Vec<_>>();

    violations.extend(nested_violations);
    violations
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<Violation> {
    pipeline
        .elements
        .iter()
        .enumerate()
        .flat_map(|(idx, element)| check_element(element, idx, pipeline, context))
        .collect()
}

fn check_block(block: &Block, context: &LintContext) -> Vec<Violation> {
    block
        .pipelines
        .iter()
        .flat_map(|pipeline| check_pipeline(pipeline, context))
        .collect()
}

fn check(context: &LintContext) -> Vec<Violation> {
    check_block(context.ast, context)
}

pub const RULE: Rule = Rule::new(
    "use_builtin_echo",
    "D not use builtin 'echo' as it's just an identity function",
    check,
    LintLevel::Warning,
)
.with_auto_fix()
.with_doc_url("https://www.nushell.sh/book/thinking_in_nu.html");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
