use std::collections::HashMap;

use crate::{
    context::LintContext,
    external_command::BuiltinAlternative,
    lint::{Fix, Replacement, RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

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

                if filter == "'.'" {
                    // jq '.' file.json -> open file.json | from json
                    if args_text.len() >= 2 {
                        format!("open {} | from json", args_text[1])
                    } else {
                        "from json".to_string()
                    }
                } else if filter.starts_with("'.") && filter.len() > 3 {
                    // jq '.field' file.json -> open file.json | from json | get field
                    let field = &filter[2..filter.len() - 1]; // Remove '. at start and ' at end
                    if args_text.len() >= 2 {
                        format!("open {} | from json | get {}", args_text[1], field)
                    } else {
                        format!("from json | get {field}")
                    }
                } else {
                    // Complex case - suggest general approach
                    "from json".to_string()
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
