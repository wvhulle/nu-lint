use std::collections::HashSet;

use nu_protocol::ast::{
    Block, Expr, Expression, Pipeline, PipelineElement, PipelineRedirection, RedirectionTarget,
    Traverse,
};

use crate::{
    Fix, Replacement, ast::span::SpanExt, context::LintContext, rule::Rule, violation::Violation,
};

fn is_dev_null_redirect(element: &PipelineElement, _context: &LintContext) -> bool {
    let Some(redirection) = &element.redirection else {
        return false;
    };

    match redirection {
        PipelineRedirection::Single { target, .. } => match target {
            RedirectionTarget::File { expr, .. } => {
                if let Expr::String(path) = &expr.expr {
                    path == "/dev/null"
                } else {
                    false
                }
            }
            RedirectionTarget::Pipe { .. } => false,
        },
        PipelineRedirection::Separate { .. } => false,
    }
}

const fn is_external_call(expr: &Expression) -> bool {
    matches!(&expr.expr, Expr::ExternalCall(..))
}

fn pipeline_has_complete(pipeline: &Pipeline, context: &LintContext) -> bool {
    use crate::ast::call::CallExt;
    pipeline.elements.iter().any(|element| {
        matches!(&element.expr.expr, Expr::Call(call)
            if call.is_call_to_command("complete", context))
    })
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Option<Violation> {
    if pipeline.elements.len() < 2 {
        return None;
    }

    let first_element = &pipeline.elements[0];

    if !is_external_call(&first_element.expr) {
        return None;
    }

    if !is_dev_null_redirect(first_element, context) {
        return None;
    }

    // Debug: log what we're checking
    log::debug!(
        "Found err> /dev/null in pipeline at span {:?}",
        first_element.expr.span
    );

    let Expr::ExternalCall(head, _args) = &first_element.expr.expr else {
        return None;
    };

    let cmd_name = head.span.text(context);

    // Check if complete is already in the pipeline
    let has_complete = pipeline_has_complete(pipeline, context);

    let message = if has_complete {
        format!(
            "External command '{cmd_name}' redirects stderr to /dev/null but already uses \
             'complete': the redirect is redundant"
        )
    } else {
        format!(
            "External command '{cmd_name}' redirects stderr to /dev/null: use 'complete' for \
             idiomatic Nushell error handling"
        )
    };

    let help = if has_complete {
        "Remove the 'err> /dev/null' redirect since 'complete' already captures stderr".to_string()
    } else {
        format!(
            "Instead of redirecting to /dev/null, use 'complete' to handle both stdout and \
             stderr:\n{cmd_name} | complete | get stdout | lines\n\nThis allows you to:\n- Access \
             stderr if needed ($result.stderr)\n- Check exit codes ($result.exit_code)\n- Handle \
             errors gracefully in Nushell"
        )
    };

    // For the violation span, use the external command expression span
    let violation_span = first_element.expr.span;

    // Build the fix
    let external_cmd_text = first_element.expr.span.text(context);
    let mut replacement_parts = vec![external_cmd_text.to_string()];

    // If complete is not already present, add it
    if !has_complete {
        replacement_parts.push("| complete | get stdout".to_string());
    }

    // Add all remaining pipeline elements
    for element in &pipeline.elements[1..] {
        replacement_parts.push("|".to_string());
        replacement_parts.push(element.expr.span.text(context).to_string());
    }

    let replacement_text = replacement_parts.join(" ");

    // Replace from start of first element to end of last element
    let pipeline_start = first_element.expr.span.start;
    let pipeline_end = pipeline
        .elements
        .last()
        .map_or(first_element.expr.span.end, |e| e.expr.span.end);
    let replace_span = nu_protocol::Span::new(pipeline_start, pipeline_end);

    log::debug!(
        "Fix spans: pipeline_start={}, pipeline_end={}, source_len={}, replace_text='{}', \
         has_complete={}",
        pipeline_start,
        pipeline_end,
        context.source_len(),
        replacement_text,
        has_complete
    );

    let fix_explanation = if has_complete {
        "Remove redundant err> /dev/null"
    } else {
        "Use complete instead of err> /dev/null"
    };

    let fix = Fix::with_explanation(
        fix_explanation,
        vec![Replacement::new(replace_span, replacement_text)],
    );

    Some(
        Violation::new(message, violation_span)
            .with_primary_label("redirect")
            .with_help(help)
            .with_fix(fix),
    )
}

fn check_block(block: &Block, context: &LintContext, violations: &mut Vec<Violation>) {
    check_block_impl(block, context, violations, &mut HashSet::new());
}

fn check_block_impl(
    block: &Block,
    context: &LintContext,
    violations: &mut Vec<Violation>,
    seen_spans: &mut HashSet<(usize, usize)>,
) {
    for pipeline in &block.pipelines {
        if let Some(violation) = check_pipeline(pipeline, context) {
            let span_key = (violation.span.start, violation.span.end);
            // Only add violation if we haven't seen this span before
            if seen_spans.insert(span_key) {
                violations.push(violation);
            }
        }

        for element in &pipeline.elements {
            let mut blocks = Vec::new();
            element.expr.flat_map(
                context.working_set,
                &|expr| match &expr.expr {
                    Expr::Block(block_id)
                    | Expr::Closure(block_id)
                    | Expr::Subexpression(block_id) => {
                        vec![*block_id]
                    }
                    _ => vec![],
                },
                &mut blocks,
            );

            for &block_id in &blocks {
                let nested_block = context.working_set.get_block(block_id);
                check_block_impl(nested_block, context, violations, seen_spans);
            }
        }
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    let mut violations = Vec::new();
    check_block(context.ast, context, &mut violations);
    violations
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_complete_over_dev_null",
        "Prefer 'complete' over redirecting stderr to /dev/null for idiomatic error handling",
        check,
    )
    .with_doc_url("https://www.nushell.sh/commands/docs/complete.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
