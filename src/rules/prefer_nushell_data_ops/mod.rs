use std::collections::HashMap;

use crate::{
    context::LintContext,
    external_command::{BuiltinAlternative, extract_external_args},
    lint::{Fix, Replacement, RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

/// Complex jq operations that can be replaced with Nushell data operations
const COMPLEX_JQ_PATTERNS: &[&str] = &[
    "map(",
    "select(",
    "group_by(",
    "sort_by(",
    "unique",
    "reverse",
    ".[]",
];

fn get_jq_data_ops() -> HashMap<&'static str, BuiltinAlternative> {
    let mut map = HashMap::new();

    map.insert(
        "jq",
        BuiltinAlternative::with_note(
            "structured data operations",
            "Use Nushell's structured data commands like 'where', 'each', 'group-by' instead of \
             jq filters",
        ),
    );

    map
}

fn build_fix(
    cmd_text: &str,
    alternative: &BuiltinAlternative,
    args: &[nu_protocol::ast::ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let args_text = extract_external_args(args, context);

    let new_text = match cmd_text {
        "jq" => {
            if args_text.is_empty() {
                alternative.command.to_string()
            } else {
                let filter = &args_text[0];

                // Common jq patterns to Nushell equivalents
                if filter.contains("map(") {
                    // jq 'map(.field)' -> each { get field }
                    "each { get field }".to_string()
                } else if filter.contains("select(") {
                    // jq 'select(.age > 30)' -> where age > 30
                    "where condition".to_string()
                } else if filter.contains("group_by(") {
                    // jq 'group_by(.category)' -> group-by category
                    "group-by field".to_string()
                } else if filter == "'.[]'" {
                    // jq '.[]' -> values (or direct iteration)
                    "values".to_string()
                } else if filter == "'sort_by(.field)'" {
                    // jq 'sort_by(.field)' -> sort-by field
                    "sort-by field".to_string()
                } else if filter.contains("unique") {
                    // jq 'unique' -> uniq
                    "uniq".to_string()
                } else if filter.contains("reverse") {
                    // jq 'reverse' -> reverse
                    "reverse".to_string()
                } else {
                    // Complex case - suggest general approach
                    "# Use structured data operations like where, each, group-by".to_string()
                }
            }
        }
        _ => alternative.command.to_string(),
    };

    Fix::new_static(
        "Replace jq filter with Nushell structured data operations",
        vec![Replacement::new_dynamic(expr_span, new_text)],
    )
}

/// Check if a jq command contains complex data operations
fn contains_complex_jq_op(source_text: &str) -> bool {
    COMPLEX_JQ_PATTERNS
        .iter()
        .any(|pattern| source_text.contains(pattern))
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    // Only detect jq commands with complex data operation patterns
    let violations = crate::external_command::detect_external_commands(
        context,
        "prefer_nushell_data_ops",
        Severity::Info,
        &get_jq_data_ops(),
        Some(build_fix),
    );

    // Filter to only show violations for complex jq operations
    violations
        .into_iter()
        .filter(|violation| {
            let source_text = &context.source[violation.span.start..violation.span.end];
            contains_complex_jq_op(source_text)
        })
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_nushell_data_ops",
        RuleCategory::Idioms,
        Severity::Info,
        "Prefer Nushell's structured data operations (where, each, group-by) over jq filters",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
