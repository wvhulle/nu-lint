use std::collections::HashSet;

use nu_protocol::{
    BlockId, Completion, Span,
    ast::{Expr, Traverse},
};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt, declaration::CustomCommandDef},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct LocatedFunction {
    definition: CustomCommandDef,
    location: Span,
}

struct UnusedFunctionFixData {
    name: String,
    removal_span: Span,
}

fn is_entry_point(f: &LocatedFunction) -> bool {
    f.definition.is_main() || f.definition.is_exported()
}

fn collect_function_definitions(context: &LintContext) -> Vec<LocatedFunction> {
    let mut functions = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            if let Expr::Call(call) = &expr.expr
                && let Some(def) = call.custom_command_def(context)
            {
                vec![LocatedFunction {
                    definition: def,
                    location: expr.span,
                }]
            } else {
                vec![]
            }
        },
        &mut functions,
    );
    functions
}

fn extract_completer_block_id(
    completion: Option<&Completion>,
    context: &LintContext,
) -> Option<BlockId> {
    if let Some(Completion::Command(decl_id)) = completion {
        let decl = context.working_set.get_decl(*decl_id);
        decl.block_id()
    } else {
        None
    }
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
                    if let Some(id) = extract_completer_block_id(param.completion.as_ref(), context)
                    {
                        ids.push(id);
                    }
                }
                for param in &sig.optional_positional {
                    if let Some(id) = extract_completer_block_id(param.completion.as_ref(), context)
                    {
                        ids.push(id);
                    }
                }
                if let Some(param) = &sig.rest_positional
                    && let Some(id) = extract_completer_block_id(param.completion.as_ref(), context)
                {
                    ids.push(id);
                }
                for flag in &sig.named {
                    if let Some(id) = extract_completer_block_id(flag.completion.as_ref(), context)
                    {
                        ids.push(id);
                    }
                }
            }
            ids
        },
        &mut block_ids,
    );
    block_ids.into_iter().collect()
}

fn collect_transitively_called_functions(
    context: &LintContext,
    entry_points: &[&LocatedFunction],
    completer_block_ids: &HashSet<BlockId>,
    all_function_block_ids: &HashSet<BlockId>,
) -> HashSet<BlockId> {
    let mut called_block_ids = HashSet::new();

    for entry in entry_points {
        let block = context.working_set.get_block(entry.definition.body);
        let transitively_called =
            block.find_transitively_called_functions(context, all_function_block_ids);
        called_block_ids.extend(transitively_called);
    }

    for completer_block_id in completer_block_ids {
        let block = context.working_set.get_block(*completer_block_id);
        let transitively_called =
            block.find_transitively_called_functions(context, all_function_block_ids);
        called_block_ids.extend(transitively_called);
    }

    called_block_ids
}

fn is_unused_function(
    f: &LocatedFunction,
    called_block_ids: &HashSet<BlockId>,
    completer_block_ids: &HashSet<BlockId>,
) -> bool {
    !f.definition.is_main()
        && !f.definition.is_exported()
        && !called_block_ids.contains(&f.definition.body)
        && !completer_block_ids.contains(&f.definition.body)
}

fn create_violation(
    context: &LintContext,
    f: &LocatedFunction,
) -> (Detection, UnusedFunctionFixData) {
    let name_span = context.find_declaration_span(&f.definition.name);
    let removal_span = context.expand_span_to_full_lines(f.location);

    let violation = Detection::from_file_span(
        format!(
            "Function '{}' is defined but never called from entry points",
            f.definition.name
        ),
        name_span,
    )
    .with_primary_label("unused function")
    .with_help(
        "Remove unused helper functions or call them from entry points (main or exported \
         functions) or other used functions",
    );

    let fix_data = UnusedFunctionFixData {
        name: f.definition.name.clone(),
        removal_span,
    };

    (violation, fix_data)
}

struct UnusedHelperFunctions;

impl DetectFix for UnusedHelperFunctions {
    type FixInput<'a> = UnusedFunctionFixData;

    fn id(&self) -> &'static str {
        "unused_helper_functions"
    }

    fn explanation(&self) -> &'static str {
        "Detect helper functions that are never called from entry points (main or exported \
         functions)"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/custom_commands.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let function_definitions = collect_function_definitions(context);

        let all_function_block_ids: HashSet<BlockId> = function_definitions
            .iter()
            .map(|f| f.definition.body)
            .collect();

        let entry_points: Vec<_> = function_definitions
            .iter()
            .filter(|f| is_entry_point(f))
            .collect();

        if entry_points.is_empty() {
            return vec![];
        }

        let completer_block_ids = collect_completer_block_ids(context);

        let called_block_ids = collect_transitively_called_functions(
            context,
            &entry_points,
            &completer_block_ids,
            &all_function_block_ids,
        );

        function_definitions
            .iter()
            .filter(|f| is_unused_function(f, &called_block_ids, &completer_block_ids))
            .map(|f| create_violation(context, f))
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
