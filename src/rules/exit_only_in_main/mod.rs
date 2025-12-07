use nu_protocol::ast::{Call, Expr};

use crate::{
    ast::{call::CallExt, span::SpanExt},
    context::LintContext,
    rule::Rule,
    violation::Violation,
};

/// Check if a call is to the 'exit' command
fn is_exit_call(call: &Call, ctx: &LintContext) -> bool {
    call.get_call_name(ctx) == "exit"
}

fn check(context: &LintContext) -> Vec<Violation> {
    // First, collect all function definitions
    let functions = context.collect_function_definitions();

    // Then, find all exit calls and check if they're in non-main functions
    context.collect_rule_violations(|expr, ctx| {
        if let Expr::Call(call) = &expr.expr {
            if !is_exit_call(call, ctx) {
                return vec![];
            }

            // Check if this exit is inside a function
            let Some(function_name) = call.head.find_containing_function(&functions, ctx) else {
                return vec![];
            };

            // Allow exit in main function
            if function_name == "main" {
                return vec![];
            }

            return vec![
                Violation::new(
                    format!(
                        "Function '{function_name}' uses 'exit' which terminates the entire script"
                    ),
                    call.head,
                )
                .with_primary_label("exit call")
                .with_extra_label(format!("inside '{function_name}'"), expr.span)
                .with_help(
                    "Use 'return' to exit the function, or 'error make' to signal an error. Only \
                     'main' should use 'exit'.",
                ),
            ];
        }
        vec![]
    })
}

pub const fn rule() -> Rule {
    Rule::new(
        "exit_only_in_main",
        "Avoid using 'exit' in functions other than 'main'",
        check,
    )
    .with_doc_url("https://www.nushell.sh/commands/docs/exit.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
