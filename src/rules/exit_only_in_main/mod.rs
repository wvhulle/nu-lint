use std::collections::HashMap;

use nu_protocol::{
    BlockId,
    ast::{Argument, Expr, Traverse},
};

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

/// Extract function definition from a call expression
fn extract_function_definition(call: &nu_protocol::ast::Call, ctx: &LintContext) -> Option<(BlockId, String)> {
    let decl = ctx.working_set.get_decl(call.decl_id);
    if !matches!(decl.name(), "def" | "export def") {
        return None;
    }
    
    // First argument is the function name
    let Argument::Positional(name_expr) = call.arguments.first()? else {
        return None;
    };
    
    let name = ctx.source.get(name_expr.span.start..name_expr.span.end)?;
    
    // Third argument is the function body block (can be Block or Closure)
    let Argument::Positional(body_expr) = call.arguments.get(2)? else {
        return None;
    };
    
    let block_id = match &body_expr.expr {
        Expr::Block(id) | Expr::Closure(id) => *id,
        _ => return None,
    };
    
    Some((block_id, name.to_string()))
}

/// Collect all function definitions with their names and block IDs
fn collect_function_definitions(ctx: &LintContext) -> HashMap<BlockId, String> {
    let mut functions = Vec::new();
    
    ctx.ast.flat_map(
        ctx.working_set,
        &|expr| {
            let Expr::Call(call) = &expr.expr else {
                return vec![];
            };
            
            extract_function_definition(call, ctx).into_iter().collect()
        },
        &mut functions,
    );
    
    functions.into_iter().collect()
}

/// Check if a span is contained within a block
fn span_in_block(
    span: nu_protocol::Span,
    block_id: BlockId,
    ctx: &LintContext,
) -> bool {
    let block = ctx.working_set.get_block(block_id);
    if let Some(block_span) = block.span {
        return span.start >= block_span.start && span.end <= block_span.end;
    }
    false
}

/// Find which function contains a given span
fn find_containing_function(
    span: nu_protocol::Span,
    functions: &HashMap<BlockId, String>,
    ctx: &LintContext,
) -> Option<String> {
    // Find the smallest (most specific) block that contains this span
    functions
        .iter()
        .filter(|(block_id, _)| span_in_block(span, **block_id, ctx))
        .min_by_key(|(block_id, _)| {
            let block = ctx.working_set.get_block(**block_id);
            block.span.map_or(usize::MAX, |s| s.end - s.start)
        })
        .map(|(_, name)| name.clone())
}

/// Check if a call is to the 'exit' command
fn is_exit_call(call: &nu_protocol::ast::Call, ctx: &LintContext) -> bool {
    let decl = ctx.working_set.get_decl(call.decl_id);
    decl.name() == "exit"
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
            let Some(function_name) = find_containing_function(call.head, &functions, ctx) else {
                return vec![];
            };
            
            // Allow exit in main function
            if function_name == "main" {
                return vec![];
            }
            
            return vec![
                RuleViolation::new_dynamic(
                    "exit_only_in_main",
                    format!("Function '{function_name}' uses 'exit' which terminates the entire script"),
                    call.head,
                )
                .with_suggestion_static(
                    "Use 'return' to exit the function, or 'error make' to signal an error. Only 'main' should use 'exit'.",
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
