use nu_protocol::ast::{Expr, Expression, Pipeline};

use crate::{
    ast::{call::CallExt, pipeline::PipelineExt},
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

/// Commands that are known to produce no output (even if type system says
/// `Type::Any`) This is a fallback for when type information is not precise
const KNOWN_NO_OUTPUT_COMMANDS: &[&str] = &[
    "cd",
    "mkdir",
    "rm",
    "mv",
    "cp",
    "touch",
    "let",
    "mut",
    "const",
    "break",
    "continue",
    "return",
    "sleep",
    "hide",
    "use",
    "source",
    "source-env",
];

/// Check if a command produces no output based on its signature's output type
/// Falls back to a whitelist for commands with `Type::Any`
fn command_produces_no_output(expr: &Expression, context: &LintContext) -> bool {
    match &expr.expr {
        Expr::Call(call) => {
            let cmd_name = call.get_call_name(context);
            let decl = context.working_set.get_decl(call.decl_id);
            let signature = decl.signature();

            // Check the output type from the signature
            let output_type = signature.get_output_type();

            match output_type {
                nu_protocol::Type::Nothing => {
                    // Definitely produces no output
                    true
                }
                nu_protocol::Type::Any => {
                    // Type system doesn't know - fall back to whitelist
                    KNOWN_NO_OUTPUT_COMMANDS.contains(&cmd_name.as_str())
                }
                _ => {
                    // Has a specific output type - produces output
                    false
                }
            }
        }
        _ => false,
    }
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Option<RuleViolation> {
    let prev_expr = pipeline.element_before_ignore(context)?;

    if !command_produces_no_output(prev_expr, context) {
        return None;
    }

    let cmd_name = match &prev_expr.expr {
        Expr::Call(call) => call.get_first_positional_arg().map_or_else(
            || {
                context
                    .working_set
                    .get_decl(call.decl_id)
                    .name()
                    .to_string()
            },
            |arg| context.source[arg.span.start..arg.span.end].to_string(),
        ),
        _ => return None,
    };

    let ignore_span = pipeline.elements.last()?.expr.span;

    Some(
        RuleViolation::new_static(
            "unnecessary_ignore",
            "Using '| ignore' with commands that produce no output",
            ignore_span,
        )
        .with_suggestion_dynamic(format!(
            "The command '{cmd_name}' produces no output, so '| ignore' is \
             unnecessary.\n\nCurrent:  {cmd_name} | ignore\nBetter:   {cmd_name}\n\nNote: \
             'ignore' only suppresses stdout. If you want to suppress errors, use 'do -i {{ ... \
             }}' instead."
        )),
    )
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context
        .ast
        .pipelines
        .iter()
        .filter_map(|pipeline| check_pipeline(pipeline, context))
        .chain(context.collect_rule_violations(|expr, ctx| {
            match &expr.expr {
                Expr::Block(block_id) | Expr::Closure(block_id) | Expr::Subexpression(block_id) => {
                    ctx.working_set
                        .get_block(*block_id)
                        .pipelines
                        .iter()
                        .filter_map(|pipeline| check_pipeline(pipeline, ctx))
                        .collect()
                }
                _ => vec![],
            }
        }))
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "unnecessary_ignore",
        RuleCategory::Idioms,
        Severity::Warning,
        "Commands that produce no output don't need '| ignore'",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
