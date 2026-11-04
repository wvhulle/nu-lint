use nu_lint_analysis::{
    Fix, LintLevel, Replacement,
    ast::block::BlockExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};
use nu_protocol::ast::{
    Expr, Expression, Pipeline, PipelineElement, PipelineRedirection, RedirectionSource,
    RedirectionTarget,
};

struct IgnoreFixData {
    replace_span: nu_protocol::Span,
    redirect_type: DevNullRedirect,
    element_spans: Vec<nu_protocol::Span>,
}

#[derive(Clone, Copy)]
enum DevNullRedirect {
    StderrOnly,
    StdoutOnly,
    Both,
}

impl DevNullRedirect {
    const fn ignore_suffix(self) -> &'static str {
        match self {
            Self::StderrOnly => "e>| ignore",
            Self::StdoutOnly => "o>| ignore",
            Self::Both => "o+e>| ignore",
        }
    }

    const fn stream_description(self) -> &'static str {
        match self {
            Self::StderrOnly => "stderr",
            Self::StdoutOnly => "stdout",
            Self::Both => "both streams",
        }
    }
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

    log::trace!(
        "Found /dev/null redirect in pipeline at span {:?}",
        first_element.expr.span
    );

    let Expr::ExternalCall(head, _args) = &first_element.expr.expr else {
        return vec![];
    };

    let cmd_name = context.expr_text(head);

    let message = format!(
        "'{cmd_name}' redirects {} to /dev/null",
        redirect_type.stream_description()
    );

    let violation_span = first_element.expr.span;

    let pipeline_start = first_element.expr.span.start;
    let pipeline_end = pipeline
        .elements
        .last()
        .map_or(first_element.expr.span.end, |e| e.expr.span.end);
    let replace_span = nu_protocol::Span::new(pipeline_start, pipeline_end);

    let violation =
        Detection::from_global_span(message, violation_span).with_primary_label("redirect");

    let fix_data = IgnoreFixData {
        replace_span,
        redirect_type,
        element_spans: pipeline
            .elements
            .iter()
            .map(|element| element.expr.span)
            .collect(),
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
        "Use `| ignore` instead of redirecting to /dev/null"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/ignore.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.ast.detect_in_pipelines(context, check_pipeline)
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let (external_span, later_spans) = fix_data.element_spans.split_first()?;
        let mut replacement_parts = vec![
            context.span_text(*external_span),
            fix_data.redirect_type.ignore_suffix(),
        ];
        for later_span in later_spans {
            replacement_parts.push("|");
            replacement_parts.push(context.span_text(*later_span));
        }
        Some(Fix {
            explanation: "Use pipe to ignore".into(),
            replacements: vec![Replacement::new(
                fix_data.replace_span,
                replacement_parts.join(" "),
            )],
        })
    }
}

pub static RULE: &dyn Rule = &IgnoreOverDevNull;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
