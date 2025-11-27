use nu_protocol::{
    Span,
    ast::{Block, Expr, Expression, Pipeline, Traverse},
};

use crate::{
    ast::call::CallExt, context::LintContext, effect::external::is_external_command_safe,
    rule::Rule, violation::Violation,
};

fn get_external_command(expr: &Expression, context: &LintContext) -> Option<String> {
    if let Expr::ExternalCall(head, _args) = &expr.expr {
        let head_text = context.source[head.span.start..head.span.end].to_string();
        if !is_external_command_safe(&head_text) {
            return Some(head_text);
        }
    }
    None
}

// Check if pipeline has any processing after the external command
// This includes both simple processing (lines, str trim) and data processing
// (from, parse, where)
const fn pipeline_has_processing(pipeline: &Pipeline, _context: &LintContext) -> bool {
    // If there's more than just the external command, it has processing
    pipeline.elements.len() > 1
}

fn is_wrapped_in_complete(pipeline: &Pipeline, context: &LintContext) -> bool {
    pipeline.elements.iter().any(|element| {
        matches!(&element.expr.expr, Expr::Call(call)
            if call.is_call_to_command("complete", context))
    })
}

fn is_in_try_block(expr_span: Span, context: &LintContext) -> bool {
    use nu_protocol::ast::Traverse;

    let mut try_spans = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            matches!(&expr.expr, Expr::Call(call)
            if call.is_call_to_command("try", context))
            .then_some(expr.span)
            .into_iter()
            .collect()
        },
        &mut try_spans,
    );

    try_spans
        .iter()
        .any(|try_span| try_span.contains_span(expr_span))
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Option<Violation> {
    // External commands in pipelines need error handling because Nushell doesn't
    // propagate errors from external commands in pipelines by default (only
    // checks the last command)
    if !pipeline_has_processing(pipeline, context) {
        return None;
    }

    let first_element = &pipeline.elements[0];
    let external_cmd = get_external_command(&first_element.expr, context)?;

    if is_wrapped_in_complete(pipeline, context) {
        return None;
    }

    // Note: try blocks do NOT catch errors from external commands in pipelines!
    // We still check for try to avoid duplicate warnings, but we should note this
    // limitation
    if is_in_try_block(first_element.expr.span, context) {
        return None;
    }

    let message = format!(
        "External command '{external_cmd}' in pipeline without error handling: Nushell only \
         checks the last command's exit code. If this command fails, the error will be silently \
         ignored."
    );

    let help = format!(
        "Wrap the external command in 'complete' to capture its exit code:\nlet result = \
         (^{external_cmd} ... | complete)\nif $result.exit_code != 0 {{\n\x20   error make {{msg: \
         $result.stderr}}\n}}\n$result.stdout"
    );

    Some(
        Violation::new(
            "prefer_complete_for_external_commands",
            message,
            first_element.expr.span,
        )
        .with_help(help),
    )
}

fn check_block(block: &Block, context: &LintContext, violations: &mut Vec<Violation>) {
    for pipeline in &block.pipelines {
        violations.extend(check_pipeline(pipeline, context));

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
                check_block(nested_block, context, violations);
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
        "prefer_complete_for_external_commands",
        "External commands in pipelines should use 'complete' for error handling (Nushell doesn't \
         propagate pipeline errors by default)",
        check,
    )
    .with_doc_url("https://www.nushell.sh/commands/docs/complete.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
