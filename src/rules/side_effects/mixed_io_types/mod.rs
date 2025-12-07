use std::collections::HashSet;

use nu_protocol::{
    BlockId,
    ast::{Block, Expr, Expression},
};

use crate::{
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    effect::builtin::{BuiltinEffect, has_builtin_side_effect},
    rule::Rule,
    violation::Violation,
};

#[derive(Debug, PartialEq, Eq, Hash)]
enum IoType {
    FileSystem,
    Network,
    Print,
}

fn collect_io_types_from_expression(
    expr: &Expression,
    context: &LintContext,
    io_types: &mut HashSet<IoType>,
) {
    match &expr.expr {
        Expr::Call(call) => {
            let command_name = call.get_call_name(context);
            if has_builtin_side_effect(&command_name, BuiltinEffect::PrintToStdout, context, call) {
                io_types.insert(IoType::Print);
            } else {
                let category = context
                    .working_set
                    .get_decl(call.decl_id)
                    .signature()
                    .category;

                match category {
                    nu_protocol::Category::FileSystem => {
                        io_types.insert(IoType::FileSystem);
                    }
                    nu_protocol::Category::Network => {
                        io_types.insert(IoType::Network);
                    }
                    _ => {}
                }
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
) -> Option<Violation> {
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
        Violation::new(message, context.find_declaration_span(function_name))
            .with_primary_label("function with mixed I/O")
            .with_help(
                "Consider separating different I/O operations into focused functions. This makes \
                 the code easier to test, mock, and reason about. Group file operations together, \
                 network operations together, and printing separately.",
            ),
    )
}

fn check(context: &LintContext) -> Vec<Violation> {
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

pub const fn rule() -> Rule {
    Rule::new(
        "mixed_io_types",
        "Functions should not mix different types of I/O operations",
        check,
    )
    .with_doc_url("https://www.nushell.sh/book/custom_commands.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
