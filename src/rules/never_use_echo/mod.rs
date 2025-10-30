use nu_protocol::ast::{Block, Expr, PipelineElement};

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

/// Check if a pipeline element uses echo (builtin or external)
/// Returns `(is_external, code_snippet)`
fn get_echo_usage(element: &PipelineElement, context: &LintContext) -> Option<(bool, String)> {
    let code_snippet = &context.source[element.expr.span.start..element.expr.span.end];

    // Check for builtin echo
    if let Expr::Call(call) = &element.expr.expr {
        let decl_name = context.working_set.get_decl(call.decl_id).name();
        if decl_name == "echo" {
            return Some((false, code_snippet.to_string()));
        }
    }

    // Check for external echo (^echo)
    if let Expr::ExternalCall(head, _args) = &element.expr.expr {
        let head_text = &context.source[head.span.start..head.span.end];
        if head_text == "echo" {
            return Some((true, code_snippet.to_string()));
        }
    }

    None
}

/// Generate a context-specific suggestion based on the echo usage
fn generate_suggestion(
    is_external: bool,
    code_snippet: &str,
    pipeline_continuation: Option<&str>,
) -> String {
    use std::fmt::Write as _;

    // Extract the argument part (everything after 'echo' or '^echo')
    let args = if is_external {
        code_snippet.strip_prefix("^echo").unwrap_or("")
    } else {
        code_snippet.strip_prefix("echo").unwrap_or("")
    }
    .trim();

    let mut suggestion = String::from("Instead of 'echo', use the value directly:\n\n");

    // Show the specific bad usage
    let _ = writeln!(suggestion, "  Bad:  {code_snippet}");

    // Suggest the direct value usage
    if args.is_empty() {
        suggestion.push_str("  Good: (no echo needed)\n\n");
    } else if let Some(continuation) = pipeline_continuation {
        // In pipeline: show the corrected version with the actual continuation
        let _ = writeln!(suggestion, "  Good: {args}\n");
        suggestion.push_str("In pipelines, echo is unnecessary - just use the value:\n");
        let _ = writeln!(suggestion, "  {args} | {continuation}\n");
    } else {
        let _ = writeln!(suggestion, "  Good: {args}\n");
    }

    // Add general guidance
    suggestion.push_str("Note:\n");
    suggestion
        .push_str("- 'echo' in Nushell is just an identity function (returns input unchanged)\n");
    suggestion.push_str("- Use 'print' if you need side-effects for debugging:\n");
    let _ = write!(suggestion, "  print {args}");

    suggestion
}

/// Get the pipeline continuation (what comes after the current element)
fn get_pipeline_continuation(
    pipeline: &nu_protocol::ast::Pipeline,
    element_idx: usize,
    context: &LintContext,
) -> Option<String> {
    if element_idx + 1 < pipeline.elements.len() {
        // Get the span from the next element to the end of the pipeline
        let start = pipeline.elements[element_idx + 1].expr.span.start;
        let end = pipeline.elements.last().unwrap().expr.span.end;
        Some(context.source[start..end].to_string())
    } else {
        None
    }
}

fn check_block(block: &Block, context: &LintContext, violations: &mut Vec<RuleViolation>) {
    for pipeline in &block.pipelines {
        for (idx, element) in pipeline.elements.iter().enumerate() {
            if let Some((is_external, code_snippet)) = get_echo_usage(element, context) {
                let message = "Avoid using 'echo' command: In Nushell, 'echo' is just an identity \
                               function. Use the value directly, or use 'print' for side-effects.";

                let pipeline_continuation = get_pipeline_continuation(pipeline, idx, context);
                let suggestion = generate_suggestion(
                    is_external,
                    &code_snippet,
                    pipeline_continuation.as_deref(),
                );

                violations.push(
                    RuleViolation::new_static("prefer_builtin_echo", message, element.expr.span)
                        .with_suggestion_dynamic(suggestion),
                );
            }

            // Check for nested blocks (functions, closures, subexpressions)
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
        Severity::Warning,
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
