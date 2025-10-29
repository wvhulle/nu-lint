use std::collections::HashMap;

use crate::{
    context::LintContext,
    external_command::{BuiltinAlternative, extract_external_args},
    lint::{Fix, Replacement, RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

#[derive(Debug, PartialEq)]
enum JqFilter<'a> {
    Identity,
    FieldAccess(&'a str),
    Complex,
}

/// Parse a jq filter to determine its type and extract field names
fn parse_jq_filter(filter: &str) -> JqFilter<'_> {
    if filter == "'.'" {
        JqFilter::Identity
    } else if filter.starts_with("'.") && filter.ends_with('\'') && filter.len() > 3 {
        let field = &filter[2..filter.len() - 1]; // Remove '. at start and ' at end
        if field.chars().all(|c| c.is_alphanumeric() || c == '_') {
            JqFilter::FieldAccess(field)
        } else {
            JqFilter::Complex
        }
    } else {
        JqFilter::Complex
    }
}

fn get_jq_alternatives() -> HashMap<&'static str, BuiltinAlternative> {
    let mut map = HashMap::new();

    map.insert(
        "jq",
        BuiltinAlternative::with_note(
            "from json",
            "Use 'from json' to parse JSON data and then use Nushell's structured data commands",
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
            // Simple jq filter patterns
            if args_text.is_empty() {
                alternative.command.to_string()
            } else {
                let filter = &args_text[0];
                let file_arg = args_text.get(1);

                match parse_jq_filter(filter) {
                    JqFilter::Identity => file_arg.map_or_else(
                        || "from json".to_string(),
                        |file| format!("open {file} | from json"),
                    ),
                    JqFilter::FieldAccess(field) => file_arg.map_or_else(
                        || format!("from json | get {field}"),
                        |file| format!("open {file} | from json | get {field}"),
                    ),
                    JqFilter::Complex => "from json".to_string(),
                }
            }
        }
        _ => alternative.command.to_string(),
    };

    Fix {
        description: format!("Replace '^{cmd_text}' with structured data processing").into(),
        replacements: vec![Replacement::new_dynamic(expr_span, new_text)],
    }
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    crate::external_command::detect_external_commands(
        context,
        "prefer_from_json",
        Severity::Info,
        &get_jq_alternatives(),
        Some(build_fix),
    )
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_from_json",
        RuleCategory::Idioms,
        Severity::Info,
        "Prefer 'from json' and structured data operations over external jq commands",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
