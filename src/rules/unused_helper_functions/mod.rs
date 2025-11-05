use std::collections::HashMap;

use crate::{
    ast::block::BlockExt,
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let function_definitions = context.collect_function_definitions();

    let function_map: HashMap<String, _> = function_definitions
        .into_iter()
        .map(|(block_id, name)| (name, block_id))
        .collect();

    let Some(&main_block_id) = function_map.get("main") else {
        return vec![];
    };

    let called_functions = main_block_id.find_transitively_called_functions(context, &function_map);

    function_map
        .keys()
        .filter(|&name| name != "main" && !called_functions.contains(name))
        .map(|name| {
            let span = context.find_declaration_span(name);
            RuleViolation::new_dynamic(
                "unused_helper_functions",
                format!("Function '{name}' is defined but never called from 'main'"),
                span,
            )
            .with_suggestion_static(
                "Remove unused helper functions or call them from 'main' or other used functions",
            )
        })
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "unused_helper_functions",
        RuleCategory::CodeQuality,
        Severity::Warning,
        "Detect helper functions that are never called in files with a 'main' function",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
