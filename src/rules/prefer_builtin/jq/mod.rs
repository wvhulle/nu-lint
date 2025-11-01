use std::collections::HashMap;

use crate::{
    context::LintContext,
    external_command::{BuiltinAlternative, extract_external_args},
    lint::{Fix, Replacement, RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

/// Simple jq operations that have direct Nushell equivalents
const SIMPLE_JQ_OPS: &[&str] = &[
    "'length'",
    "'keys'",
    "'type'",
    "'empty'",
    "'not'",
    "'flatten'",
    "'add'",
    "'min'",
    "'max'",
    "'sort'",
    "'unique'",
];

fn get_simple_jq_alternatives() -> HashMap<&'static str, BuiltinAlternative> {
    let mut map = HashMap::new();

    map.insert(
        "jq",
        BuiltinAlternative::with_note(
            "built-in commands",
            "Use built-in Nushell commands for simple operations - they're faster and more \
             idiomatic",
        ),
    );

    map
}

fn format_jq_replacement(filter: &str, file_arg: Option<&str>) -> String {
    let with_file = |cmd: &str| {
        file_arg.map_or_else(
            || cmd.to_string(),
            |file| format!("open {file} | from json | {cmd}"),
        )
    };

    match filter {
        "'length'" => with_file("length"),
        "'keys'" => with_file("columns"),
        "'type'" => with_file("describe"),
        "'empty'" => "null".to_string(),
        "'not'" => "not".to_string(),
        "'flatten'" => "flatten".to_string(),
        "'add'" => "math sum".to_string(),
        "'min'" => "math min".to_string(),
        "'max'" => "math max".to_string(),
        "'sort'" => "sort".to_string(),
        "'unique'" => "uniq".to_string(),
        _ if filter.starts_with("'.[") && filter.ends_with("]'") => {
            let index = &filter[3..filter.len() - 2];
            format!("get {index}")
        }
        _ => "# Use structured data operations".to_string(),
    }
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
                let file_arg = args_text.get(1).map(String::as_str);
                format_jq_replacement(filter, file_arg)
            }
        }
        _ => alternative.command.to_string(),
    };

    Fix::new_static(
        "Replace simple jq operation with built-in Nushell command",
        vec![Replacement::new_dynamic(expr_span, new_text)],
    )
}

/// Check if a jq command contains simple operations
fn contains_simple_jq_op(source_text: &str) -> bool {
    SIMPLE_JQ_OPS.iter().any(|op| source_text.contains(op))
        || (source_text.contains("'.[") && source_text.contains("]'"))
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    // Only detect jq commands with simple operations that have direct Nushell
    // equivalents
    let violations = crate::external_command::detect_external_commands(
        context,
        "avoid_jq_for_simple_ops",
        &get_simple_jq_alternatives(),
        Some(build_fix),
    );

    // Filter to only show violations for simple jq operations
    violations
        .into_iter()
        .filter(|violation| {
            let source_text = &context.source[violation.span.start..violation.span.end];
            contains_simple_jq_op(source_text)
        })
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "avoid_jq_for_simple_ops",
        RuleCategory::Performance,
        Severity::Warning,
        "Avoid jq for simple operations that have direct Nushell built-in equivalents",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
