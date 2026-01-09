use nu_protocol::ast::{Block, Call, Expr, Pipeline, PipelineElement};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn extract_print_message(call: &Call, context: &LintContext) -> Option<String> {
    call.get_first_positional_arg()
        .map(|expr| expr.span_text(context).to_string())
}

fn extract_exit_code(call: &Call) -> Option<i64> {
    call.get_first_positional_arg()
        .and_then(|code_expr| match &code_expr.expr {
            Expr::Int(code) => Some(*code),
            _ => None,
        })
}

struct ErrorToStdout {
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
            let print_message = extract_print_message(print_call, context)?;
            let exit_code = extract_exit_code(exit_call)?;
            (exit_code != 0).then_some(print_message)
        })
        .map(|_print_message| ErrorToStdout {
            span: print_call.span(),
        })
}

fn check_sequential_print_exit(
    first: &PipelineElement,
    second: &PipelineElement,
    context: &LintContext,
) -> Option<ErrorToStdout> {
    let Expr::Call(print_call) = &first.expr.expr else {
        return None;
    };
    let Expr::Call(exit_call) = &second.expr.expr else {
        return None;
    };
    check_print_exit_calls(print_call, exit_call, context)
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

            let Expr::Call(print_call) = &first.expr.expr else {
                return None;
            };
            let Expr::Call(exit_call) = &second.expr.expr else {
                return None;
            };
            check_print_exit_calls(print_call, exit_call, context)
        })
    })
}

fn create_violation(pattern: &ErrorToStdout) -> Detection {
    Detection::from_global_span(
        "Error message printed to stdout instead of stderr",
        pattern.span,
    )
    .with_primary_label("prints error to stdout")
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
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "errors_to_stderr"
    }

    fn short_description(&self) -> &'static str {
        "Error messages should go to stderr, not stdout"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/print.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
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
