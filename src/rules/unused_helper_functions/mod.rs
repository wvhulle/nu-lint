use std::collections::HashSet;

use nu_protocol::{
    BlockId, Completion, Span,
    ast::{Expr, Traverse},
};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct FunctionDef {
    name: String,
    block_id: BlockId,
    def_span: Span,
    is_exported: bool,
}

struct UnusedFunctionFixData {
    name: String,
    removal_span: Span,
}

fn is_main_entry_point(name: &str) -> bool {
    name == "main" || name.starts_with("main ")
}

fn collect_function_definitions(context: &LintContext) -> Vec<FunctionDef> {
    let mut functions = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            if let Expr::Call(call) = &expr.expr
                && let Some(def) = call.custom_command_def(context)
            {
                let is_exported = def.is_exported();
                vec![FunctionDef {
                    name: def.name,
                    block_id: def.body,
                    def_span: expr.span,
                    is_exported,
                }]
            } else {
                vec![]
            }
        },
        &mut functions,
    );
    functions
}

fn collect_completer_block_ids(context: &LintContext) -> HashSet<BlockId> {
    let mut block_ids = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            let mut ids = Vec::new();
            if let Expr::Call(call) = &expr.expr
                && let Some(def) = call.custom_command_def(context)
            {
                let block = context.working_set.get_block(def.body);
                let sig = &block.signature;

                for param in &sig.required_positional {
                    if let Some(Completion::Command(decl_id)) = &param.completion {
                        let decl = context.working_set.get_decl(*decl_id);
                        if let Some(block_id) = decl.block_id() {
                            ids.push(block_id);
                        }
                    }
                }
                for param in &sig.optional_positional {
                    if let Some(Completion::Command(decl_id)) = &param.completion {
                        let decl = context.working_set.get_decl(*decl_id);
                        if let Some(block_id) = decl.block_id() {
                            ids.push(block_id);
                        }
                    }
                }
                if let Some(param) = &sig.rest_positional
                    && let Some(Completion::Command(decl_id)) = &param.completion
                {
                    let decl = context.working_set.get_decl(*decl_id);
                    if let Some(block_id) = decl.block_id() {
                        ids.push(block_id);
                    }
                }
                for flag in &sig.named {
                    if let Some(Completion::Command(decl_id)) = &flag.completion {
                        let decl = context.working_set.get_decl(*decl_id);
                        if let Some(block_id) = decl.block_id() {
                            ids.push(block_id);
                        }
                    }
                }
            }
            ids
        },
        &mut block_ids,
    );
    block_ids.into_iter().collect()
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
        let function_definitions = collect_function_definitions(context);

        // Build set of all function block IDs for transitive call analysis
        let all_function_block_ids: HashSet<BlockId> =
            function_definitions.iter().map(|f| f.block_id).collect();

        // Find entry points (main functions)
        let entry_points: Vec<_> = function_definitions
            .iter()
            .filter(|f| is_main_entry_point(&f.name))
            .collect();

        if entry_points.is_empty() {
            return vec![];
        }

        // Collect block IDs of functions used as completers
        let completer_block_ids = collect_completer_block_ids(context);

        // Trace all transitively called functions from entry points
        let mut called_block_ids = HashSet::new();
        for entry in &entry_points {
            let block = context.working_set.get_block(entry.block_id);
            let transitively_called =
                block.find_transitively_called_functions(context, &all_function_block_ids);
            called_block_ids.extend(transitively_called);
        }

        function_definitions
            .iter()
            .filter(|f| {
                !is_main_entry_point(&f.name)
                    && !f.is_exported
                    && !called_block_ids.contains(&f.block_id)
                    && !completer_block_ids.contains(&f.block_id)
            })
            .map(|f| {
                let name_span = context.find_declaration_span(&f.name);
                let removal_span = context.expand_span_to_full_lines(f.def_span);

                let violation = Detection::from_file_span(
                    format!(
                        "Function '{}' is defined but never called from 'main'",
                        f.name
                    ),
                    name_span,
                )
                .with_primary_label("unused function")
                .with_help(
                    "Remove unused helper functions or call them from 'main' or other used \
                     functions",
                );

                let fix_data = UnusedFunctionFixData {
                    name: f.name.clone(),
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
