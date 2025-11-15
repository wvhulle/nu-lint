use nu_protocol::ast::{Block, Call, Expr, Pipeline, PipelineElement};

use crate::{
    ast::{call::CallExt, expression::ExpressionExt, span::SpanExt},
    context::LintContext,
    rule::Rule,
    violation::Violation,
};

struct PrintExitPattern {
    print_message: String,
    exit_code: i64,
    span: nu_protocol::Span,
}

fn check_print_exit_calls(
    print_call: &Call,
    exit_call: &Call,
    context: &LintContext,
) -> Option<PrintExitPattern> {
    (print_call.is_call_to_command("print", context) && !print_call.has_named_flag("stderr"))
        .then(|| exit_call.is_call_to_command("exit", context))
        .filter(|&is_exit| is_exit)
        .and_then(|_| {
            let print_message = print_call.extract_print_message(context)?;
            let exit_code = exit_call.extract_exit_code()?;
            (exit_code != 0).then_some((print_message, exit_code))
        })
        .map(|(print_message, exit_code)| PrintExitPattern {
            print_message,
            exit_code,
            span: print_call.span().merge(exit_call.span()),
        })
}

fn check_sequential_print_exit(
    first: &PipelineElement,
    second: &PipelineElement,
    context: &LintContext,
) -> Option<PrintExitPattern> {
    first
        .expr
        .extract_call()
        .zip(second.expr.extract_call())
        .and_then(|(print_call, exit_call)| check_print_exit_calls(print_call, exit_call, context))
}

fn check_same_pipeline_print_exit(
    pipeline: &Pipeline,
    context: &LintContext,
) -> Option<PrintExitPattern> {
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

fn extract_first_function_parameter(
    context: &LintContext,
    span: nu_protocol::Span,
) -> Option<String> {
    context
        .ast
        .pipelines
        .iter()
        .flat_map(|pipeline| &pipeline.elements)
        .filter_map(|element| element.expr.extract_call())
        .find_map(|call| {
            let (_, _) = call.extract_function_definition(context)?;
            let block_id = call.get_positional_arg(2)?.extract_block_id()?;
            let func_block = context.working_set.get_block(block_id);

            func_block
                .span
                .filter(|s| s.contains_span(span))
                .and_then(|_| {
                    func_block
                        .signature
                        .required_positional
                        .first()
                        .and_then(|param| param.var_id)
                        .map(|var_id| {
                            context
                                .working_set
                                .get_variable(var_id)
                                .declaration_span
                                .text(context)
                                .to_string()
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

fn build_suggestion(pattern: &PrintExitPattern, context: &LintContext) -> String {
    let truncated_msg = truncate_message(&pattern.print_message, 60);

    let example_span = extract_first_function_parameter(context, pattern.span).map_or_else(
        || "$span".to_string(),
        |param| format!("(metadata ${param}).span"),
    );

    format!(
        "Replace print + exit pattern with error make for better error handling.\n\nCurrent \
         code:\n\x20 print \"{truncated_msg}\"\n\x20 exit {}\n\nSuggested replacement:\n\x20 \
         error make {{\n\x20   msg: \"{truncated_msg}\"\n\x20   label: {{\n\x20     text: \"error \
         occurred here\"\n\x20     span: {example_span}\n\x20   }}\n\x20   help: \"describe how \
         to fix this issue\"\n\x20 }}",
        pattern.exit_code
    )
}

fn create_violation(pattern: &PrintExitPattern, context: &LintContext) -> Violation {
    let suggestion = build_suggestion(pattern, context);

    Violation::new_static(
        "print_exit_use_error_make",
        "Use 'error make' instead of 'print' + 'exit' for error conditions",
        pattern.span,
    )
    .with_suggestion_dynamic(suggestion)
}

fn check_sequential_patterns<'a>(
    block: &'a Block,
    context: &'a LintContext,
) -> impl Iterator<Item = PrintExitPattern> + 'a {
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
) -> impl Iterator<Item = PrintExitPattern> + 'a {
    block
        .pipelines
        .iter()
        .filter_map(move |pipeline| check_same_pipeline_print_exit(pipeline, context))
}

fn check_block_patterns(block: &Block, context: &LintContext) -> Vec<Violation> {
    check_same_pipeline_patterns(block, context)
        .chain(check_sequential_patterns(block, context))
        .map(|pattern| create_violation(&pattern, context))
        .collect()
}

fn check(context: &LintContext) -> Vec<Violation> {
    let main_violations = check_block_patterns(context.ast, context);

    let nested_violations: Vec<_> = context.collect_rule_violations(|expr, ctx| match &expr.expr {
        Expr::Closure(block_id) | Expr::Block(block_id) | Expr::Subexpression(block_id) => {
            let block = ctx.working_set.get_block(*block_id);
            check_block_patterns(block, ctx)
        }
        _ => vec![],
    });

    main_violations
        .into_iter()
        .chain(nested_violations)
        .collect()
}

pub const fn rule() -> Rule {
    Rule::new(
        "print_exit_use_error_make",
        "Replace 'print' + 'exit' patterns with 'error make' for proper error handling",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
