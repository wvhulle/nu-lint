use nu_protocol::ast::{
    Block, Expr, Expression, Pipeline, PipelineElement, PipelineRedirection, RedirectionSource,
    RedirectionTarget, Traverse,
};

use crate::{
    Fix, LintLevel, Replacement, ast::span::SpanExt, context::LintContext, rule::Rule,
    violation::Violation,
};

enum DevNullRedirect {
    StderrOnly,
    StdoutOnly,
    Both,
}

fn detect_dev_null_redirect(element: &PipelineElement) -> Option<DevNullRedirect> {
    let redirection = element.redirection.as_ref()?;

    match redirection {
        PipelineRedirection::Single { source, target } => match target {
            RedirectionTarget::File { expr, .. } => {
                if let Expr::String(path) = &expr.expr
                    && path == "/dev/null"
                {
                    return Some(match source {
                        RedirectionSource::Stdout => DevNullRedirect::StdoutOnly,
                        RedirectionSource::Stderr => DevNullRedirect::StderrOnly,
                        RedirectionSource::StdoutAndStderr => DevNullRedirect::Both,
                    });
                }
                None
            }
            RedirectionTarget::Pipe { .. } => None,
        },
        PipelineRedirection::Separate { out, err } => {
            let out_is_dev_null = matches!(out, RedirectionTarget::File { expr, .. }
                if matches!(&expr.expr, Expr::String(p) if p == "/dev/null"));
            let err_is_dev_null = matches!(err, RedirectionTarget::File { expr, .. }
                if matches!(&expr.expr, Expr::String(p) if p == "/dev/null"));

            match (out_is_dev_null, err_is_dev_null) {
                (true, true) => Some(DevNullRedirect::Both),
                (true, false) => Some(DevNullRedirect::StdoutOnly),
                (false, true) => Some(DevNullRedirect::StderrOnly),
                (false, false) => None,
            }
        }
    }
}

const fn is_external_call(expr: &Expression) -> bool {
    matches!(&expr.expr, Expr::ExternalCall(..))
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Option<Violation> {
    let first_element = &pipeline.elements[0];

    if !is_external_call(&first_element.expr) {
        return None;
    }

    let redirect_type = detect_dev_null_redirect(first_element)?;

    log::debug!(
        "Found /dev/null redirect in pipeline at span {:?}",
        first_element.expr.span
    );

    let Expr::ExternalCall(head, _args) = &first_element.expr.expr else {
        return None;
    };

    let cmd_name = head.span.source_code(context);

    let (message, replacement_suffix) = match redirect_type {
        DevNullRedirect::StderrOnly => (
            format!("'{cmd_name}' redirects stderr to /dev/null"),
            "e>| ignore",
        ),
        DevNullRedirect::StdoutOnly => (
            format!("'{cmd_name}' redirects stdout to /dev/null"),
            "o>| ignore",
        ),
        DevNullRedirect::Both => (
            format!("'{cmd_name}' redirects both streams to /dev/null"),
            "o+e>| ignore",
        ),
    };

    let violation_span = first_element.expr.span;

    let external_cmd_text = first_element.expr.span.source_code(context);
    let mut replacement_parts = vec![
        external_cmd_text.to_string(),
        replacement_suffix.to_string(),
    ];

    for element in &pipeline.elements[1..] {
        replacement_parts.push("|".to_string());
        replacement_parts.push(element.expr.span.source_code(context).to_string());
    }

    let replacement_text = replacement_parts.join(" ");

    let pipeline_start = first_element.expr.span.start;
    let pipeline_end = pipeline
        .elements
        .last()
        .map_or(first_element.expr.span.end, |e| e.expr.span.end);
    let replace_span = nu_protocol::Span::new(pipeline_start, pipeline_end);

    log::debug!(
        "Fix: pipeline_start={pipeline_start}, pipeline_end={pipeline_end}, \
         replace_text='{replacement_text}'"
    );

    let fix = Fix::with_explanation(
        "Use pipe to ignore",
        vec![Replacement::new(replace_span, replacement_text)],
    );

    Some(
        Violation::new(message, violation_span)
            .with_primary_label("redirect")
            .with_fix(fix),
    )
}

fn check_block(block: &Block, context: &LintContext, violations: &mut Vec<Violation>) {
    for pipeline in &block.pipelines {
        if let Some(violation) = check_pipeline(pipeline, context) {
            violations.push(violation);
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

pub const RULE: Rule = Rule::new(
    "ignore_over_dev_null",
    "Use '| ignore' instead of redirecting to /dev/null",
    check,
    LintLevel::Warning,
)
.with_auto_fix()
.with_doc_url("https://www.nushell.sh/commands/docs/ignore.html");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
