use nu_protocol::{
    Span,
    ast::{Argument, Block, Expr, ExternalArgument, Pipeline, PipelineElement},
};

use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

/// Semantic fix data: stores the span of the echo call and its arguments
pub struct FixData {
    /// Full span of the echo call expression
    element_span: Span,
    /// Span covering all arguments (None if no arguments)
    args_span: Option<Span>,
}

const fn external_arg_span(arg: &ExternalArgument) -> Span {
    match arg {
        ExternalArgument::Regular(expr) | ExternalArgument::Spread(expr) => expr.span,
    }
}

/// Extract the span of arguments from an echo call
fn extract_echo_args_span(element: &PipelineElement, context: &LintContext) -> Option<Span> {
    match &element.expr.expr {
        Expr::Call(call) => {
            if !call.is_call_to_command("echo", context) {
                return None;
            }
            // Get spans of all positional arguments
            let arg_spans: Vec<Span> = call
                .arguments
                .iter()
                .filter_map(|arg| match arg {
                    Argument::Positional(expr)
                    | Argument::Unknown(expr)
                    | Argument::Spread(expr) => Some(expr.span),
                    Argument::Named(_) => None,
                })
                .collect();

            if arg_spans.is_empty() {
                None
            } else {
                // Merge all argument spans into one
                let start = arg_spans.iter().map(|s| s.start).min()?;
                let end = arg_spans.iter().map(|s| s.end).max()?;
                Some(Span::new(start, end))
            }
        }
        Expr::ExternalCall(head, args) => {
            if context.expr_text(head) != "echo" {
                return None;
            }
            // For external calls, get the span of arguments
            let arg_spans: Vec<Span> = args.iter().map(external_arg_span).collect();

            if arg_spans.is_empty() {
                None
            } else {
                let start = arg_spans.iter().map(|s| s.start).min()?;
                let end = arg_spans.iter().map(|s| s.end).max()?;
                Some(Span::new(start, end))
            }
        }
        _ => None,
    }
}

fn uses_echo(element: &PipelineElement, context: &LintContext) -> bool {
    match &element.expr.expr {
        Expr::Call(call) => call.is_call_to_command("echo", context),
        Expr::ExternalCall(head, _) => context.expr_text(head) == "echo",
        _ => false,
    }
}

fn extract_nested_block_ids(
    element: &PipelineElement,
    context: &LintContext,
) -> Vec<nu_protocol::BlockId> {
    use nu_protocol::ast::Traverse;

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
    blocks
}

fn detect_element(
    element: &PipelineElement,
    _idx: usize,
    _pipeline: &Pipeline,
    context: &LintContext,
) -> Vec<(Detection, FixData)> {
    let mut violations = Vec::new();

    if uses_echo(element, context) {
        let message = "Avoid 'echo' - it's just an identity function. Use the value directly, or \
                       'print' for debugging";
        let violation = Detection::from_global_span(message, element.expr.span);
        let args_span = extract_echo_args_span(element, context);
        let fix_data = FixData {
            element_span: element.expr.span,
            args_span,
        };
        violations.push((violation, fix_data));
    }

    let nested_violations = extract_nested_block_ids(element, context)
        .iter()
        .flat_map(|&block_id| {
            let block = context.working_set.get_block(block_id);
            detect_block(block, context)
        })
        .collect::<Vec<_>>();

    violations.extend(nested_violations);
    violations
}

fn detect_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<(Detection, FixData)> {
    pipeline
        .elements
        .iter()
        .enumerate()
        .flat_map(|(idx, element)| detect_element(element, idx, pipeline, context))
        .collect()
}

fn detect_block(block: &Block, context: &LintContext) -> Vec<(Detection, FixData)> {
    block
        .pipelines
        .iter()
        .flat_map(|pipeline| detect_pipeline(pipeline, context))
        .collect()
}

struct UseBuiltinEcho;

impl DetectFix for UseBuiltinEcho {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "echo_just_identity"
    }

    fn short_description(&self) -> &'static str {
        "Do not use the built-in (or external) 'echo' as it's just an identity function in Nushell."
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/thinking_in_nu.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let block: &Block = context.ast;
        block
            .pipelines
            .iter()
            .flat_map(|pipeline| detect_pipeline(pipeline, context))
            .collect()
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let args_span = fix_data.args_span?;
        let args_text = context.span_text(args_span);

        let code_snippet = context.span_text(fix_data.element_span);
        Some(Fix::with_explanation(
            format!("Replace '{code_snippet}' with '{args_text}'"),
            vec![Replacement::new(
                fix_data.element_span,
                args_text.to_string(),
            )],
        ))
    }
}

pub static RULE: &dyn Rule = &UseBuiltinEcho;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
