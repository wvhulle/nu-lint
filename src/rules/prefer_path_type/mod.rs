use crate::{
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

const PATH_KEYWORDS: &[&str] = &["path", "file", "dir", "directory", "folder", "location"];

const FALSE_POSITIVE_KEYWORDS: &[&str] = &[
    "xpath",
    "jsonpath",
    "class_path",
    "classpath",
    "java_path",
    "python_path",
    "import_path",
];

fn is_likely_filesystem_path(param_name: &str) -> bool {
    let lower_name = param_name.to_lowercase();

    FALSE_POSITIVE_KEYWORDS
        .iter()
        .all(|&fp| !lower_name.contains(fp))
        && lower_name
            .split(|c: char| c == '_' || c == '-' || !c.is_alphanumeric())
            .any(|part| {
                PATH_KEYWORDS
                    .iter()
                    .any(|&kw| part == kw || part.ends_with(kw))
            })
}

fn check_parameter(
    param: &nu_protocol::PositionalArg,
    function_name: &str,
    function_span: nu_protocol::Span,
    is_optional: bool,
) -> Option<RuleViolation> {
    use nu_protocol::SyntaxShape::{Any, String as StringShape};

    matches!(param.shape, StringShape | Any)
        .then_some(())
        .filter(|()| is_likely_filesystem_path(&param.name))
        .map(|()| {
            let (current_type, message) = match param.shape {
                StringShape => (
                    "string",
                    format!(
                        "Parameter `{}` in function `{function_name}` appears to be a path but \
                         uses `string` type",
                        param.name
                    ),
                ),
                Any => (
                    "no type annotation",
                    format!(
                        "Parameter `{}` in function `{function_name}` appears to be a path but \
                         has no type annotation",
                        param.name
                    ),
                ),
                _ => unreachable!(),
            };

            let suggestion = format!(
                "Change parameter `{}` type from `{current_type}` to `path`:\n  {}{}:  path",
                param.name,
                param.name,
                if is_optional { "?" } else { "" }
            );

            RuleViolation::new_dynamic("prefer_path_type", message, function_span)
                .with_suggestion_dynamic(suggestion)
        })
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context
        .collect_function_definitions()
        .iter()
        .flat_map(|(block_id, function_name)| {
            let block = context.working_set.get_block(*block_id);
            let function_span = context.find_declaration_span(function_name);

            block
                .signature
                .required_positional
                .iter()
                .filter_map(move |param| {
                    check_parameter(param, function_name, function_span, false)
                })
                .chain(
                    block
                        .signature
                        .optional_positional
                        .iter()
                        .filter_map(move |param| {
                            check_parameter(param, function_name, function_span, true)
                        }),
                )
                .chain(
                    block
                        .signature
                        .rest_positional
                        .iter()
                        .filter_map(move |param| {
                            check_parameter(param, function_name, function_span, false)
                        }),
                )
                .collect::<Vec<_>>()
        })
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_path_type",
        RuleCategory::TypeSafety,
        Severity::Warning,
        "Use Nushell's path type instead of string for parameters with 'path' in the name",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
