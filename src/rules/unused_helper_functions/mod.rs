use std::collections::{HashMap, HashSet};

use nu_protocol::{
    BlockId, Span,
    ast::{Expr, Traverse},
};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct UnusedFunctionFixData {
    name: String,
    removal_span: Span,
}

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
                && let Some(def) = call.custom_command_def(context)
            {
                vec![(def.name, (def.body, expr.span))]
            } else {
                vec![]
            }
        },
        &mut functions,
    );
    functions.into_iter().collect()
}

struct UnusedHelperFunctions;

impl DetectFix for UnusedHelperFunctions {
    type FixInput<'a> = UnusedFunctionFixData;

    fn id(&self) -> &'static str {
        "unused_helper_functions"
    }

    fn explanation(&self) -> &'static str {
        "Detect helper functions that are never called in files with a 'main' function"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/custom_commands.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
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
            let transitively_called =
                block.find_transitively_called_functions(context, &function_map);
            called_functions.extend(transitively_called);
        }

        function_definitions
            .iter()
            .filter(|(name, _)| !is_main_entry_point(name) && !called_functions.contains(*name))
            .map(|(name, (_, def_span))| {
                let name_span = context.find_declaration_span(name);
                let removal_span = context.expand_span_to_full_lines(*def_span);

                let violation = Detection::from_file_span(
                    format!("Function '{name}' is defined but never called from 'main'"),
                    name_span,
                )
                .with_primary_label("unused function")
                .with_help(
                    "Remove unused helper functions or call them from 'main' or other used \
                     functions",
                );

                let fix_data = UnusedFunctionFixData {
                    name: name.clone(),
                    removal_span,
                };

                (violation, fix_data)
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix::with_explanation(
            format!("Remove unused function '{}'", fix_data.name),
            vec![Replacement::new(fix_data.removal_span, String::new())],
        ))
    }
}

pub static RULE: &dyn Rule = &UnusedHelperFunctions;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
