use nu_protocol::ast::{
    Expr, Expression, Pipeline, PipelineElement, PipelineRedirection, RedirectionSource,
    RedirectionTarget,
};

use crate::{
    Fix, LintLevel, Replacement,
    ast::block::BlockExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct IgnoreFixData {
    replace_span: nu_protocol::Span,
    replacement_text: String,
}

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

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<(Detection, IgnoreFixData)> {
    let Some(first_element) = pipeline.elements.first() else {
        return vec![];
    };

    if !is_external_call(&first_element.expr) {
        return vec![];
    }

    let Some(redirect_type) = detect_dev_null_redirect(first_element) else {
        return vec![];
    };

    log::debug!(
        "Found /dev/null redirect in pipeline at span {:?}",
        first_element.expr.span
    );

    let Expr::ExternalCall(head, _args) = &first_element.expr.expr else {
        return vec![];
    };

    let cmd_name = context.expr_text(head);

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

    let external_cmd_text = context.expr_text(&first_element.expr);
    let mut replacement_parts = vec![
        external_cmd_text.to_string(),
        replacement_suffix.to_string(),
    ];

    for element in &pipeline.elements[1..] {
        replacement_parts.push("|".to_string());
        replacement_parts.push(context.expr_text(&element.expr).to_string());
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

    let violation =
        Detection::from_global_span(message, violation_span).with_primary_label("redirect");

    let fix_data = IgnoreFixData {
        replace_span,
        replacement_text,
    };

    vec![(violation, fix_data)]
}

struct IgnoreOverDevNull;

impl DetectFix for IgnoreOverDevNull {
    type FixInput<'a> = IgnoreFixData;

    fn id(&self) -> &'static str {
        "ignore_over_dev_null"
    }

    fn short_description(&self) -> &'static str {
        "Use '| ignore' instead of redirecting to /dev/null"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/ignore.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.ast.detect_in_pipelines(context, check_pipeline)
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix::with_explanation(
            "Use pipe to ignore",
            vec![Replacement::new(
                fix_data.replace_span,
                fix_data.replacement_text.clone(),
            )],
        ))
    }
}

pub static RULE: &dyn Rule = &IgnoreOverDevNull;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
