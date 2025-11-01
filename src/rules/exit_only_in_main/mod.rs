use std::collections::HashMap;

use nu_protocol::{BlockId, ast::{Expr, Traverse}};

use crate::{
    ast_utils::{AstUtils, BlockUtils, DeclarationUtils},
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

/// Collect all function definitions with their names and block IDs
fn collect_function_definitions(ctx: &LintContext) -> HashMap<BlockId, String> {
    let mut functions = Vec::new();

    ctx.ast.flat_map(
        ctx.working_set,
        &|expr| {
            let Expr::Call(call) = &expr.expr else {
                return vec![];
            };

            DeclarationUtils::extract_function_definition(&call, ctx).into_iter().collect()
        },
        &mut functions,
    );

    functions.into_iter().collect()
}

/// Check if a call is to the 'exit' command
fn is_exit_call(call: &nu_protocol::ast::Call, ctx: &LintContext) -> bool {
    AstUtils::get_call_name(call, ctx) == "exit"
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    // First, collect all function definitions
    let functions = collect_function_definitions(context);

    // Then, find all exit calls and check if they're in non-main functions
    context.collect_rule_violations(|expr, ctx| {
        if let Expr::Call(call) = &expr.expr {
            if !is_exit_call(call, ctx) {
                return vec![];
            }

            // Check if this exit is inside a function
            let Some(function_name) = BlockUtils::find_containing_function(call.head, &functions, ctx) else {
                return vec![];
            };

            // Allow exit in main function
            if function_name == "main" {
                return vec![];
            }

            return vec![
                RuleViolation::new_dynamic(
                    "exit_only_in_main",
                    format!(
                        "Function '{function_name}' uses 'exit' which terminates the entire script"
                    ),
                    call.head,
                )
                .with_suggestion_static(
                    "Use 'return' to exit the function, or 'error make' to signal an error. Only \
                     'main' should use 'exit'.",
                ),
            ];
        }
        vec![]
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "exit_only_in_main",
        RuleCategory::CodeQuality,
        Severity::Error,
        "Avoid using 'exit' in functions other than 'main'",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
