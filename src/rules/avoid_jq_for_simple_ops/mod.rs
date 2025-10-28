use std::collections::HashMap;

use crate::{
    context::LintContext,
    external_command::BuiltinAlternative,
    lint::{Fix, Replacement, RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

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

                match filter.as_str() {
                    "'length'" => {
                        // jq 'length' -> length
                        if args_text.len() >= 2 {
                            format!("open {} | from json | length", args_text[1])
                        } else {
                            "length".to_string()
                        }
                    }
                    "'keys'" => {
                        // jq 'keys' -> columns
                        if args_text.len() >= 2 {
                            format!("open {} | from json | columns", args_text[1])
                        } else {
                            "columns".to_string()
                        }
                    }
                    "'type'" => {
                        // jq 'type' -> describe
                        if args_text.len() >= 2 {
                            format!("open {} | from json | describe", args_text[1])
                        } else {
                            "describe".to_string()
                        }
                    }
                    "'empty'" => {
                        // jq 'empty' -> null or empty
                        "null".to_string()
                    }
                    "'not'" => {
                        // jq 'not' -> not
                        "not".to_string()
                    }
                    "'flatten'" => {
                        // jq 'flatten' -> flatten
                        "flatten".to_string()
                    }
                    "'add'" => {
                        // jq 'add' -> math sum (for arrays)
                        "math sum".to_string()
                    }
                    "'min'" => {
                        // jq 'min' -> math min
                        "math min".to_string()
                    }
                    "'max'" => {
                        // jq 'max' -> math max
                        "math max".to_string()
                    }
                    _ => {
                        // Check for simple array indexing
                        if filter.starts_with("'.[") && filter.ends_with("]'") {
                            let index = &filter[3..filter.len() - 2];
                            format!("get {index}")
                        } else {
                            "# Use structured data operations".to_string()
                        }
                    }
                }
            }
        }
        _ => alternative.command.to_string(),
    };

    Fix::new_static(
        "Replace simple jq operation with built-in Nushell command",
        vec![Replacement::new_dynamic(expr_span, new_text)],
    )
}

fn extract_external_args(
    args: &[nu_protocol::ast::ExternalArgument],
    context: &LintContext,
) -> Vec<String> {
    args.iter()
        .map(|arg| match arg {
            nu_protocol::ast::ExternalArgument::Regular(expr) => {
                context.source[expr.span.start..expr.span.end].to_string()
            }
            nu_protocol::ast::ExternalArgument::Spread(expr) => {
                format!("...{}", &context.source[expr.span.start..expr.span.end])
            }
        })
        .collect()
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    // Only detect jq commands with simple operations that have direct Nushell
    // equivalents
    let violations = crate::external_command::detect_external_commands(
        context,
        "avoid_jq_for_simple_ops",
        Severity::Warning,
        &get_simple_jq_alternatives(),
        Some(build_fix),
    );

    // Filter to only show violations for simple jq operations
    violations
        .into_iter()
        .filter(|violation| {
            let source_text = &context.source[violation.span.start..violation.span.end];
            // Simple operations that have direct equivalents
            source_text.contains("'length'")
                || source_text.contains("'keys'")
                || source_text.contains("'type'")
                || source_text.contains("'empty'")
                || source_text.contains("'not'")
                || source_text.contains("'flatten'")
                || source_text.contains("'add'")
                || source_text.contains("'min'")
                || source_text.contains("'max'")
                || (source_text.contains("'.[") && source_text.contains("]'"))
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
