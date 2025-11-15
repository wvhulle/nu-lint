use std::collections::HashSet;

use nu_protocol::{
    BlockId,
    ast::{Block, Expr, Expression},
};

use crate::{
    LintLevel,
    ast::{
        call::CallExt,
        effect::{IoType, get_io_type},
        expression::ExpressionExt,
    },
    context::LintContext,
    rule::Rule,
    violation::RuleViolation,
};

fn collect_io_types_from_expression(
    expr: &Expression,
    context: &LintContext,
    io_types: &mut HashSet<IoType>,
) {
    match &expr.expr {
        Expr::Call(call) => {
            let command_name = call.get_call_name(context);
            if let Some(io_type) = get_io_type(&command_name, context, call) {
                io_types.insert(io_type);
            }

            for arg_expr in call.all_arg_expressions() {
                if let Some(block_id) = arg_expr.extract_block_id() {
                    let block = context.working_set.get_block(block_id);
                    collect_io_types_from_block(block, context, io_types);
                } else {
                    collect_io_types_from_expression(arg_expr, context, io_types);
                }
            }
        }
        Expr::ExternalCall(_, _) => {
            io_types.insert(IoType::FileSystem);
        }
        _ => {}
    }
}

fn collect_io_types_from_block(
    block: &Block,
    context: &LintContext,
    io_types: &mut HashSet<IoType>,
) {
    for pipeline in &block.pipelines {
        for element in &pipeline.elements {
            collect_io_types_from_expression(&element.expr, context, io_types);
        }
    }
}

fn analyze_function_body(
    block_id: BlockId,
    function_name: &str,
    context: &LintContext,
) -> Option<RuleViolation> {
    let block = context.working_set.get_block(block_id);

    let mut io_types = HashSet::new();
    collect_io_types_from_block(block, context, &mut io_types);

    if io_types.len() < 2 {
        return None;
    }

    let io_type_names: Vec<&str> = io_types
        .iter()
        .map(|t| match t {
            IoType::FileSystem => "file I/O",
            IoType::Network => "network I/O",
            IoType::Print => "print",
        })
        .collect();

    let message = format!(
        "Function `{function_name}` mixes different I/O types: {}",
        io_type_names.join(", ")
    );

    Some(
        RuleViolation::new_dynamic(
            "mixed_io_types",
            message,
            context.find_declaration_span(function_name),
        )
        .with_suggestion_static(
            "Consider separating different I/O operations into focused functions. This makes the \
             code easier to test, mock, and reason about. Group file operations together, network \
             operations together, and printing separately.",
        ),
    )
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let function_definitions = context.collect_function_definitions();

    let has_main = function_definitions.values().any(|name| name == "main");
    if !has_main {
        return vec![];
    }

    function_definitions
        .iter()
        .filter(|(_, name)| *name != "main")
        .filter(|(_, name)| !context.is_exported_function(name))
        .filter_map(|(block_id, name)| analyze_function_body(*block_id, name, context))
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "mixed_io_types",
        LintLevel::Allow,
        "Functions should not mix different types of I/O operations",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
