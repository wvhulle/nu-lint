use nu_protocol::{
    Span,
    ast::{Block, Expr, Pipeline},
};

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

/// Check if a pipeline is an alias or export definition
fn is_alias_or_export_definition(pipeline: &Pipeline, context: &LintContext) -> bool {
    // Check if the pipeline starts with 'alias' or 'export' command
    if let Some(first_element) = pipeline.elements.first()
        && let Expr::Call(call) = &first_element.expr.expr
    {
        let decl_name = context.working_set.get_decl(call.decl_id).name();
        log::debug!("Pipeline first element is a Call to: {}", decl_name);
        let is_alias_or_export = decl_name == "alias"
            || decl_name == "export"
            || decl_name == "export alias"
            || decl_name == "export def"
            || decl_name == "export const"
            || decl_name == "export use"
            || decl_name == "export extern"
            || decl_name == "def"
            || decl_name == "const";
        log::debug!("Is alias or export? {is_alias_or_export}");
        return is_alias_or_export;
    }

    log::debug!("Pipeline first element is NOT a Call");
    false
}

/// Check if a pipeline contains an external command
fn pipeline_contains_external(pipeline: &Pipeline, context: &LintContext) -> Option<Span> {
    use nu_protocol::ast::Traverse;

    let mut external_span = None;
    pipeline.elements.iter().for_each(|element| {
        if external_span.is_some() {
            return;
        }

        let mut found = Vec::new();
        element.expr.flat_map(
            context.working_set,
            &|expr| {
                if let Expr::ExternalCall(_head, _args) = &expr.expr {
                    vec![expr.span]
                } else {
                    vec![]
                }
            },
            &mut found,
        );

        if let Some(&span) = found.first() {
            external_span = Some(span);
        }
    });

    external_span
}

/// Check a block for sequential external commands
fn check_block(block: &Block, context: &LintContext, violations: &mut Vec<RuleViolation>) {
    let pipelines: Vec<&Pipeline> = block.pipelines.iter().collect();
    log::debug!("Checking block with {} pipelines", pipelines.len());

    for window in pipelines.windows(2) {
        let [first_pipeline, second_pipeline] = window else {
            continue;
        };

        log::debug!("Checking pair of pipelines");

        // Skip if either pipeline is an alias or export definition
        if is_alias_or_export_definition(first_pipeline, context)
            || is_alias_or_export_definition(second_pipeline, context)
        {
            log::debug!("Skipping - one or both pipelines are alias/export definitions");
            continue;
        }

        let (Some(first_span), Some(second_span)) = (
            pipeline_contains_external(first_pipeline, context),
            pipeline_contains_external(second_pipeline, context),
        ) else {
            log::debug!("Skipping - one or both pipelines don't contain external commands");
            continue;
        };

        log::debug!(
            "Found sequential external commands at spans: {first_span:?} and {second_span:?}"
        );

        if let Some(violation) = create_violation_if_needed(first_span, second_span, context) {
            log::debug!("Creating violation");
            violations.push(violation);
        }
    }

    // Recursively check nested blocks
    for pipeline in &block.pipelines {
        check_pipeline_for_nested_blocks(pipeline, context, violations);
    }
}

/// Check a pipeline for nested blocks and recursively check them
fn check_pipeline_for_nested_blocks(
    pipeline: &Pipeline,
    context: &LintContext,
    violations: &mut Vec<RuleViolation>,
) {
    use nu_protocol::ast::Traverse;

    for element in &pipeline.elements {
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

        for block_id in blocks {
            let block = context.working_set.get_block(block_id);
            check_block(block, context, violations);
        }
    }
}

/// Get a substring from the end, ensuring we don't split multi-byte characters
fn safe_suffix(text: &str, max_bytes: usize) -> &str {
    if text.len() <= max_bytes {
        text
    } else {
        let start_idx = text.len().saturating_sub(max_bytes);
        let boundary_idx = (start_idx..text.len())
            .find(|&i| text.is_char_boundary(i))
            .unwrap_or(text.len());
        &text[boundary_idx..]
    }
}

/// Get a substring from the start, ensuring we don't split multi-byte characters
fn safe_prefix(text: &str, max_bytes: usize) -> &str {
    if text.len() <= max_bytes {
        text
    } else {
        let end_idx = (0..=max_bytes)
            .rev()
            .find(|&i| text.is_char_boundary(i))
            .unwrap_or(0);
        &text[..end_idx]
    }
}

/// Check if an external command is wrapped in error handling
fn is_wrapped_in_error_handling(span: Span, context: &LintContext) -> bool {
    const CONTEXT_SIZE: usize = 100;

    let source_before = &context.source[..span.start];
    let source_after = &context.source[span.end..];

    let prefix_text = safe_suffix(source_before, CONTEXT_SIZE);
    let suffix_text = safe_prefix(source_after, CONTEXT_SIZE);

    (prefix_text.contains("try {") || prefix_text.contains("try{"))
        || suffix_text.trim_start().starts_with("| complete")
        || suffix_text.contains("| complete)")
}

/// Check if text contains error handling patterns between commands
fn has_error_handling_between(text: &str) -> bool {
    text.contains("exit_code")
        || text.contains("LAST_EXIT_CODE")
        || text.contains("try")
        || text.contains("&&")
        || text.contains("complete")
}

fn create_violation_if_needed(
    first_span: Span,
    second_span: Span,
    context: &LintContext,
) -> Option<RuleViolation> {
    // First check if either command is wrapped in error handling
    if is_wrapped_in_error_handling(first_span, context)
        || is_wrapped_in_error_handling(second_span, context)
    {
        return None;
    }

    // Then check for error handling between the commands
    let between_text = &context.source[first_span.end..second_span.start];
    if has_error_handling_between(between_text) {
        return None;
    }

    // Use the first command's span for the violation location
    Some(
        RuleViolation::new_static(
            "sequential_external_commands",
            "Sequential external commands without error checking - failures in first command \
             ignored",
            first_span,
        )
        .with_suggestion_static(
            "Check exit codes using 'try', 'complete', or check $env.LAST_EXIT_CODE between \
             commands",
        ),
    )
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let mut violations = Vec::new();
    check_block(context.ast, context, &mut violations);
    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "sequential_external_commands",
        RuleCategory::ErrorHandling,
        Severity::Warning,
        "Ensure sequential external commands have error checking between them",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
