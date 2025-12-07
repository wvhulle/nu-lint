use std::collections::{HashMap, HashSet};

use nu_protocol::{
    BlockId, Span,
    ast::{Expr, Traverse},
};

use crate::{
    ast::{block::BlockExt, call::CallExt},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement, Violation},
};

fn is_main_entry_point(name: &str) -> bool {
    name == "main" || name.starts_with("main ")
}

fn collect_function_definitions_with_spans(
    context: &LintContext,
) -> HashMap<String, (BlockId, Span)> {
    let mut functions = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            if let Expr::Call(call) = &expr.expr
                && let Some((block_id, name)) = call.extract_function_definition(context)
            {
                vec![(name, (block_id, expr.span))]
            } else {
                vec![]
            }
        },
        &mut functions,
    );
    functions.into_iter().collect()
}

fn expand_span_to_full_line(span: Span, source: &str) -> Span {
    let bytes = source.as_bytes();

    let start = bytes[..span.start]
        .iter()
        .rposition(|&b| b == b'\n')
        .map_or(0, |pos| pos + 1);

    let end = bytes[span.end..]
        .iter()
        .position(|&b| b == b'\n')
        .map_or(source.len(), |pos| span.end + pos + 1);

    Span::new(start, end)
}

fn check(context: &LintContext) -> Vec<Violation> {
    let function_definitions = collect_function_definitions_with_spans(context);

    let function_map: HashMap<String, BlockId> = function_definitions
        .iter()
        .map(|(name, (block_id, _))| (name.clone(), *block_id))
        .collect();

    let entry_points: Vec<_> = function_definitions
        .iter()
        .filter(|(name, _)| is_main_entry_point(name))
        .collect();

    if entry_points.is_empty() {
        return vec![];
    }

    let mut called_functions = HashSet::new();
    for (_, (block_id, _)) in &entry_points {
        let block = context.working_set.get_block(*block_id);
        let transitively_called = block.find_transitively_called_functions(context, &function_map);
        called_functions.extend(transitively_called);
    }

    function_definitions
        .iter()
        .filter(|(name, _)| !is_main_entry_point(name) && !called_functions.contains(*name))
        .map(|(name, (_, def_span))| {
            let name_span = context.find_declaration_span(name);
            let removal_span = expand_span_to_full_line(*def_span, context.source);

            let fix = Fix::with_explanation(
                format!("Remove unused function '{name}'"),
                vec![Replacement::new(removal_span, String::new())],
            );

            Violation::new(
                format!("Function '{name}' is defined but never called from 'main'"),
                name_span,
            )
            .with_primary_label("unused function")
            .with_help(
                "Remove unused helper functions or call them from 'main' or other used functions",
            )
            .with_fix(fix)
        })
        .collect()
}

pub const fn rule() -> Rule {
    Rule::new(
        "unused_helper_functions",
        "Detect helper functions that are never called in files with a 'main' function",
        check,
    )
    .with_doc_url("https://www.nushell.sh/book/custom_commands.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
