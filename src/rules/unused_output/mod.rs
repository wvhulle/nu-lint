use nu_protocol::ast::{Expr, Expression, Pipeline};

use crate::{
    ast::{call::CallExt, pipeline::PipelineExt, span::SpanExt},
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

/// Commands that are known to produce output but may have `Type::Any` in their
/// signature
const FALLBACK_KNOWN_BUILTIN_OUTPUT_COMMANDS: &[&str] = &[
    "print",
    "ls",
    "http get",
    "http post",
    "open",
    "cat",
    "parse",
    "from json",
    "from csv",
    "to json",
    "to csv",
    "select",
    "where",
    "get",
    "find",
    "each",
    "reduce",
    "sort-by",
    "group-by",
    "echo",
];

const KNOWN_EXTERNAL_OUTPUT_COMMANDS: &[&str] = &[
    "echo", "ls", "cat", "find", "grep", "curl", "wget", "head", "tail", "sort",
    "whoami",   // Prints the effective username
    "hostname", // Prints the system's hostname
    "pwd",      // Prints the present working directory
    "tty",      // Prints the file name of the terminal connected to stdin
    "id",       // Prints real and effective user and group IDs
    "who",      // Prints who is logged on (always lists at least the current session)
    "date",     // Prints the current date and time
    "uptime",   // Prints system uptime and load
    "uname",    // Prints system information (e.g., "Linux", "Darwin")
    "df",       // Prints filesystem disk space usage (always lists mounts)
    "ps",       // Prints process status (at least lists itself and the parent shell)
    "echo",     // Prints its arguments. `echo` alone prints a newline.
    "history",  // Prints the command history
];

const KNOWN_EXTERNAL_NO_OUTPUT_COMMANDS: &[&str] = &[
    "cd", "mkdir", "rm", "mv", "cp", "touch", "exit", "clear", "ln",    // Creates a link
    "chmod", // Changes file permissions
    "chown", // Changes file ownership
    "chgrp", // Changes file group
    "kill",  // Sends a signal to a process (like SIGTERM)
    "sleep", // Pauses execution for a set time
];

/// Check if a command produces output based on its signature's output type
/// Falls back to a whitelist for commands with `Type::Any`
fn command_produces_output(expr: &Expression, context: &LintContext) -> bool {
    match &expr.expr {
        Expr::ExternalCall(call, _) => {
            let cmd_name = call.span.text(context);

            KNOWN_EXTERNAL_OUTPUT_COMMANDS.contains(&cmd_name)
                || !KNOWN_EXTERNAL_NO_OUTPUT_COMMANDS.contains(&cmd_name)
        }
        Expr::Call(call) => {
            let cmd_name = call.get_call_name(context);

            let decl = context.working_set.get_decl(call.decl_id);
            let signature = decl.signature();

            // Check the output type from the signature
            let output_type = signature.get_output_type();

            match output_type {
                nu_protocol::Type::Nothing => {
                    // Definitely produces no output
                    false
                }
                nu_protocol::Type::Any => {
                    // Type system doesn't know - fall back to whitelist
                    log::debug!("Command '{cmd_name}' has output type Any, checking whitelist");
                    FALLBACK_KNOWN_BUILTIN_OUTPUT_COMMANDS.contains(&cmd_name.as_str())
                }
                _ => {
                    // Has a specific output type (String, List, etc.) - produces output
                    log::debug!("Command '{cmd_name}' has output type: {output_type:?}");
                    true
                }
            }
        }
        _ => false,
    }
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Option<RuleViolation> {
    let prev_expr = pipeline.element_before_ignore(context)?;

    if !command_produces_output(prev_expr, context) {
        return None;
    }

    let prev_call = match &prev_expr.expr {
        Expr::Call(call) => call.get_call_name(context),
        _ => "pipeline".to_string(),
    };

    let ignore_span = pipeline.elements.last()?.expr.span;

    Some(
        RuleViolation::new_static(
            "unused_output",
            "Discarding command output with '| ignore'",
            ignore_span,
        )
        .with_suggestion_dynamic(format!(
            "Command '{prev_call}' produces output that is being discarded with '| ignore'.\n\nIf \
             you don't need the output, consider:\n1. Removing the command if it has no side \
             effects\n2. Using error handling if you only care about success/failure:\n   try {{ \
             {prev_call} }}\n3. If the output is intentionally discarded, add a comment \
             explaining why"
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
        "unused_output",
        RuleCategory::Idioms,
        Severity::Warning,
        "Commands producing output that is discarded with '| ignore'",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
