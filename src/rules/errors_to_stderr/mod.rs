use nu_protocol::ast::{Block, Call, Expr, Pipeline, PipelineElement};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct ErrorToStdout {
    print_message: String,
    span: nu_protocol::Span,
}

fn check_print_exit_calls(
    print_call: &Call,
    exit_call: &Call,
    context: &LintContext,
) -> Option<ErrorToStdout> {
    (print_call.is_call_to_command("print", context) && !print_call.has_named_flag("stderr"))
        .then(|| exit_call.is_call_to_command("exit", context))
        .filter(|&is_exit| is_exit)
        .and_then(|_| {
            let print_message = print_call.extract_print_message(context)?;
            let exit_code = exit_call.extract_exit_code()?;
            (exit_code != 0).then_some(print_message)
        })
        .map(|print_message| ErrorToStdout {
            print_message,
            span: print_call.span(),
        })
}

fn check_sequential_print_exit(
    first: &PipelineElement,
    second: &PipelineElement,
    context: &LintContext,
) -> Option<ErrorToStdout> {
    first
        .expr
        .extract_call()
        .zip(second.expr.extract_call())
        .and_then(|(print_call, exit_call)| check_print_exit_calls(print_call, exit_call, context))
}

fn check_same_pipeline_print_exit(
    pipeline: &Pipeline,
    context: &LintContext,
) -> Option<ErrorToStdout> {
    (pipeline.elements.len() >= 2).then_some(()).and_then(|()| {
        pipeline.elements.windows(2).find_map(|elements| {
            let [first, second] = elements else {
                return None;
            };

            first
                .expr
                .extract_call()
                .zip(second.expr.extract_call())
                .and_then(|(print_call, exit_call)| {
                    check_print_exit_calls(print_call, exit_call, context)
                })
        })
    })
}

fn truncate_message(msg: &str, max_len: usize) -> String {
    if msg.len() > max_len {
        format!("{}...", &msg[..max_len])
    } else {
        msg.to_string()
    }
}

fn create_violation(pattern: &ErrorToStdout) -> Detection {
    let truncated_msg = truncate_message(&pattern.print_message, 60);

    Detection::from_global_span(
        "Error message printed to stdout instead of stderr",
        pattern.span,
    )
    .with_primary_label("prints error to stdout")
    .with_help(format!(
        "Use 'print --stderr \"{truncated_msg}\"' to send error messages to stderr"
    ))
}

fn check_sequential_patterns<'a>(
    block: &'a Block,
    context: &'a LintContext,
) -> impl Iterator<Item = ErrorToStdout> + 'a {
    block.pipelines.windows(2).filter_map(move |pipelines| {
        let [first_pipeline, second_pipeline] = pipelines else {
            return None;
        };

        let [first_elem] = &first_pipeline.elements[..] else {
            return None;
        };
        let [second_elem] = &second_pipeline.elements[..] else {
            return None;
        };

        check_sequential_print_exit(first_elem, second_elem, context)
    })
}

fn check_same_pipeline_patterns<'a>(
    block: &'a Block,
    context: &'a LintContext,
) -> impl Iterator<Item = ErrorToStdout> + 'a {
    block
        .pipelines
        .iter()
        .filter_map(move |pipeline| check_same_pipeline_print_exit(pipeline, context))
}

fn check_block_patterns(block: &Block, context: &LintContext) -> Vec<Detection> {
    check_same_pipeline_patterns(block, context)
        .chain(check_sequential_patterns(block, context))
        .map(|pattern| create_violation(&pattern))
        .collect()
}

struct ErrorsToStderr;

impl DetectFix for ErrorsToStderr {
    type FixInput = ();

    fn id(&self) -> &'static str {
        "errors_to_stderr"
    }

    fn explanation(&self) -> &'static str {
        "Error messages should go to stderr, not stdout"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/print.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
        let main_violations = check_block_patterns(context.ast, context);

        let nested_violations: Vec<_> = context.detect(|expr, ctx| match &expr.expr {
            Expr::Closure(block_id) | Expr::Block(block_id) | Expr::Subexpression(block_id) => {
                let block = ctx.working_set.get_block(*block_id);
                check_block_patterns(block, ctx)
            }
            _ => vec![],
        });

        Self::no_fix(
            main_violations
                .into_iter()
                .chain(nested_violations)
                .collect(),
        )
    }
}

pub static RULE: &dyn Rule = &ErrorsToStderr;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
