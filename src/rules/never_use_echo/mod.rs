use nu_protocol::ast::{Block, Expr, PipelineElement};

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

/// Check if a pipeline element uses echo (builtin or external)
fn uses_echo(element: &PipelineElement, context: &LintContext) -> bool {
    match &element.expr.expr {
        Expr::Call(call) => context.working_set.get_decl(call.decl_id).name() == "echo",
        Expr::ExternalCall(head, _) => &context.source[head.span.start..head.span.end] == "echo",
        _ => false,
    }
}

/// Generate a context-specific suggestion based on the echo usage
fn generate_suggestion(
    element: &PipelineElement,
    pipeline_continuation: Option<&str>,
    context: &LintContext,
) -> String {
    use std::fmt::Write as _;

    let code_snippet = &context.source[element.expr.span.start..element.expr.span.end];

    // Extract arguments (everything after 'echo' or '^echo')
    let args = code_snippet
        .strip_prefix("^echo")
        .or_else(|| code_snippet.strip_prefix("echo"))
        .unwrap_or("")
        .trim();

    let mut suggestion = String::from("Instead of 'echo', use the value directly:\n\n");
    let _ = writeln!(suggestion, "  Bad:  {code_snippet}");

    if args.is_empty() {
        suggestion.push_str("  Good: (no echo needed)\n\n");
    } else {
        let _ = writeln!(suggestion, "  Good: {args}\n");
        if let Some(continuation) = pipeline_continuation {
            suggestion.push_str("In pipelines, echo is unnecessary - just use the value:\n");
            let _ = writeln!(suggestion, "  {args} | {continuation}\n");
        }
    }

    suggestion.push_str("Note:\n");
    suggestion
        .push_str("- 'echo' in Nushell is just an identity function (returns input unchanged)\n");
    suggestion.push_str("- Use 'print' if you need side-effects for debugging:\n");
    let _ = write!(suggestion, "  print {args}");

    suggestion
}

/// Extract what comes after the current element in a pipeline
fn get_pipeline_continuation<'a>(
    pipeline: &nu_protocol::ast::Pipeline,
    element_idx: usize,
    context: &'a LintContext,
) -> Option<&'a str> {
    (element_idx + 1 < pipeline.elements.len()).then(|| {
        let start = pipeline.elements[element_idx + 1].expr.span.start;
        let end = pipeline.elements.last().unwrap().expr.span.end;
        &context.source[start..end]
    })
}

fn check_block(block: &Block, context: &LintContext, violations: &mut Vec<RuleViolation>) {
    for pipeline in &block.pipelines {
        for (idx, element) in pipeline.elements.iter().enumerate() {
            if !uses_echo(element, context) {
                check_element_for_nested_blocks(element, context, violations);
                continue;
            }

            let message = "Avoid using 'echo' command: In Nushell, 'echo' is just an identity \
                           function. Use the value directly, or use 'print' for side-effects.";

            let pipeline_continuation = get_pipeline_continuation(pipeline, idx, context);
            let suggestion = generate_suggestion(element, pipeline_continuation, context);

            violations.push(
                RuleViolation::new_static("prefer_builtin_echo", message, element.expr.span)
                    .with_suggestion_dynamic(suggestion),
            );

            check_element_for_nested_blocks(element, context, violations);
        }
    }
}

fn check_element_for_nested_blocks(
    element: &PipelineElement,
    context: &LintContext,
    violations: &mut Vec<RuleViolation>,
) {
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

    for &block_id in &blocks {
        let block = context.working_set.get_block(block_id);
        check_block(block, context, violations);
    }
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let mut violations = Vec::new();
    check_block(context.ast, context, &mut violations);
    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "never_use_echo",
        RuleCategory::Idioms,
        Severity::Error,
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
