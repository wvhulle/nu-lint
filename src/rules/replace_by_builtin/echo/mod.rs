use nu_protocol::ast::{Block, Expr, Pipeline, PipelineElement};

use crate::{ast::call::CallExt, context::LintContext, rule::Rule, violation::Violation};

fn uses_echo(element: &PipelineElement, context: &LintContext) -> bool {
    match &element.expr.expr {
        Expr::Call(call) => call.is_call_to_command("echo", context),
        Expr::ExternalCall(head, _) => &context.source[head.span.start..head.span.end] == "echo",
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

fn format_suggestion_body(args: &str, pipeline_continuation: Option<&str>) -> String {
    use std::fmt::Write as _;

    let mut body = String::new();

    if args.is_empty() {
        body.push_str("  Good: (no echo needed)\n\n");
    } else {
        let _ = writeln!(body, "  Good: {args}\n");
        if let Some(continuation) = pipeline_continuation {
            body.push_str("In pipelines, echo is unnecessary - just use the value:\n");
            let _ = writeln!(body, "  {args} | {continuation}\n");
        }
    }

    body
}

fn format_suggestion_note(args: &str) -> String {
    use std::fmt::Write as _;

    let mut note = String::from("Note:\n");
    note.push_str("- 'echo' in Nushell is just an identity function (returns input unchanged)\n");
    note.push_str("- Use 'print' if you need side-effects for debugging:\n");
    let _ = write!(note, "  print {args}");

    note
}

fn generate_suggestion(
    element: &PipelineElement,
    pipeline_continuation: Option<&str>,
    context: &LintContext,
) -> String {
    use std::fmt::Write as _;

    let code_snippet = &context.source[element.expr.span.start..element.expr.span.end];
    let args = extract_echo_args(code_snippet);

    let mut suggestion = String::from("Instead of 'echo', use the value directly:\n\n");
    let _ = writeln!(suggestion, "  Bad:  {code_snippet}");
    suggestion.push_str(&format_suggestion_body(args, pipeline_continuation));
    suggestion.push_str(&format_suggestion_note(args));

    suggestion
}

fn get_pipeline_continuation<'a>(
    pipeline: &Pipeline,
    element_idx: usize,
    context: &'a LintContext,
) -> Option<&'a str> {
    pipeline.elements.get(element_idx + 1).map(|next_element| {
        let start = next_element.expr.span.start;
        let end = pipeline.elements.last().unwrap().expr.span.end;
        &context.source[start..end]
    })
}

fn create_violation(
    element: &PipelineElement,
    pipeline_continuation: Option<&str>,
    context: &LintContext,
) -> Violation {
    let message = "Avoid using 'echo' command: In Nushell, 'echo' is just an identity function. \
                   Use the value directly, or use 'print' for side-effects.";
    let suggestion = generate_suggestion(element, pipeline_continuation, context);

    Violation::new_static("prefer_builtin_echo", message, element.expr.span)
        .with_suggestion_dynamic(suggestion)
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

pub fn rule() -> Rule {
    Rule::new(
        "never_use_echo",
        "Discourage use of builtin 'echo' command as it's just an identity function",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
