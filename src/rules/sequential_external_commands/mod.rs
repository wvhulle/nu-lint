use nu_protocol::{
    Span,
    ast::{Block, Expr, Pipeline},
};

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn is_alias_or_export_definition(pipeline: &Pipeline, context: &LintContext) -> bool {
    pipeline
        .elements
        .first()
        .and_then(|element| {
            if let Expr::Call(call) = &element.expr.expr {
                let decl_name = context.working_set.get_decl(call.decl_id).name();
                log::debug!("Pipeline first element is a Call to: {decl_name}");
                Some(matches!(
                    decl_name,
                    "alias"
                        | "export"
                        | "export alias"
                        | "export def"
                        | "export const"
                        | "export use"
                        | "export extern"
                        | "def"
                        | "const"
                ))
            } else {
                log::debug!("Pipeline first element is NOT a Call");
                None
            }
        })
        .unwrap_or(false)
}

fn pipeline_contains_external(pipeline: &Pipeline, context: &LintContext) -> Option<Span> {
    use nu_protocol::ast::Traverse;

    pipeline.elements.iter().find_map(|element| {
        let mut found = Vec::new();
        element.expr.flat_map(
            context.working_set,
            &|expr| {
                if let Expr::ExternalCall(head, _args) = &expr.expr {
                    let head_text = &context.source[head.span.start..head.span.end];
                    log::debug!("Found ExternalCall: head={:?}, head.expr={:?} at span {:?}",
                               head_text, head.expr, expr.span);

                    // Check if this is a known built-in command by checking all decls in engine_state
                    // get_decls_sorted returns all declarations including built-ins
                    let is_known_command = context.engine_state
                        .get_decls_sorted(false)
                        .iter()
                        .any(|(name, _id)| name == head_text.as_bytes());

                    log::debug!("Command {:?} is known built-in/user-defined: {}", head_text, is_known_command);

                    if is_known_command {
                        log::debug!("Skipping - command {:?} is a known built-in or user-defined command", head_text);
                        vec![]
                    } else {
                        log::debug!("Confirmed as truly external command");
                        vec![expr.span]
                    }
                } else {
                    vec![]
                }
            },
            &mut found,
        );
        if let Some(span) = found.first().copied() {
            log::debug!("Pipeline contains external command at span {:?}", span);
            Some(span)
        } else {
            None
        }
    })
}

fn check_block(block: &Block, context: &LintContext, violations: &mut Vec<RuleViolation>) {
    log::debug!("Checking block with {} pipelines", block.pipelines.len());

    block
        .pipelines
        .windows(2)
        .filter_map(|window| {
            let [first_pipeline, second_pipeline] = window else {
                return None;
            };

            log::debug!("Checking pair of pipelines");

            if is_alias_or_export_definition(first_pipeline, context)
                || is_alias_or_export_definition(second_pipeline, context)
            {
                log::debug!("Skipping - one or both pipelines are alias/export definitions");
                return None;
            }

            let first_span = pipeline_contains_external(first_pipeline, context)?;
            let second_span = pipeline_contains_external(second_pipeline, context)?;

            log::debug!(
                "Found sequential external commands at spans: {first_span:?} and {second_span:?}"
            );

            create_violation_if_needed(
                first_span,
                second_span,
                first_pipeline,
                second_pipeline,
                context,
            )
        })
        .for_each(|violation| {
            log::debug!("Creating violation");
            violations.push(violation);
        });

    block
        .pipelines
        .iter()
        .for_each(|pipeline| check_pipeline_for_nested_blocks(pipeline, context, violations));
}

/// Check a pipeline for nested blocks and recursively check them
fn check_pipeline_for_nested_blocks(
    pipeline: &Pipeline,
    context: &LintContext,
    violations: &mut Vec<RuleViolation>,
) {
    use nu_protocol::ast::Traverse;

    pipeline.elements.iter().for_each(|element| {
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
    });
}

fn is_in_try_block(expr_span: Span, context: &LintContext) -> bool {
    use nu_protocol::ast::Traverse;

    let mut try_spans = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            matches!(&expr.expr, Expr::Call(call)
            if context.working_set.get_decl(call.decl_id).name() == "try")
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

fn pipeline_has_complete(pipeline: &Pipeline, context: &LintContext) -> bool {
    pipeline.elements.iter().any(|element| {
        matches!(&element.expr.expr, Expr::Call(call)
            if context.working_set.get_decl(call.decl_id).name() == "complete")
    })
}

fn is_wrapped_in_error_handling(span: Span, pipeline: &Pipeline, context: &LintContext) -> bool {
    if is_in_try_block(span, context) {
        log::debug!("Command is in try block");
        return true;
    }

    if pipeline_has_complete(pipeline, context) {
        log::debug!("Pipeline contains complete");
        return true;
    }

    false
}

fn has_error_handling_between(first_span: Span, second_span: Span, context: &LintContext) -> bool {
    use nu_protocol::ast::Traverse;

    let between_span = Span::new(first_span.end, second_span.start);
    let mut conditionals = Vec::new();

    context.ast.flat_map(
        context.working_set,
        &|expr| {
            (between_span.contains_span(expr.span)
                && matches!(&expr.expr, Expr::Call(call)
                    if matches!(context.working_set.get_decl(call.decl_id).name(), "if" | "match")))
            .then_some(())
            .into_iter()
            .collect()
        },
        &mut conditionals,
    );

    if !conditionals.is_empty() {
        log::debug!("Found conditional between commands");
        return true;
    }

    let between_text = &context.source[first_span.end..second_span.start];
    let has_exit_code = between_text.contains("exit_code")
        || between_text.contains("LAST_EXIT_CODE")
        || between_text.contains("&&");

    if has_exit_code {
        log::debug!("Found exit code check pattern in text");
    }

    has_exit_code
}

fn create_violation_if_needed(
    first_span: Span,
    second_span: Span,
    first_pipeline: &Pipeline,
    second_pipeline: &Pipeline,
    context: &LintContext,
) -> Option<RuleViolation> {
    if is_wrapped_in_error_handling(first_span, first_pipeline, context)
        || is_wrapped_in_error_handling(second_span, second_pipeline, context)
        || has_error_handling_between(first_span, second_span, context)
    {
        return None;
    }

    Some(
        RuleViolation::new_static(
            "sequential_external_commands",
            "Sequential external commands without error checking - failures in first command ignored",
            first_span,
        )
        .with_suggestion_static(
            "Check exit codes using 'try', 'complete', or check $env.LAST_EXIT_CODE between commands",
        )
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
