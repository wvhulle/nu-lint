use crate::{
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

const DURATION_KEYWORDS: &[&str] = &[
    "timeout", "duration", "delay", "interval", "period", "wait", "sleep", "elapsed", "time",
];

const FALSE_POSITIVE_KEYWORDS: &[&str] = &[
    "timestamp",
    "datetime",
    "timezone",
    "timer",
    "timed",
    "timing",
    "longtime",
    "daytime",
    "nighttime",
    "runtime",
    "compile_time",
    "compiletime",
];

fn is_likely_duration(param_name: &str) -> bool {
    let lower_name = param_name.to_lowercase();

    FALSE_POSITIVE_KEYWORDS
        .iter()
        .all(|&fp| !lower_name.contains(fp))
        && lower_name
            .split(|c: char| c == '_' || c == '-' || !c.is_alphanumeric())
            .any(|part| {
                DURATION_KEYWORDS
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
        .filter(|()| is_likely_duration(&param.name))
        .map(|()| {
            let (current_type, message) = match param.shape {
                StringShape => (
                    "string",
                    format!(
                        "Parameter `{}` in function `{function_name}` appears to be a duration \
                         but uses `string` type",
                        param.name
                    ),
                ),
                Any => (
                    "no type annotation",
                    format!(
                        "Parameter `{}` in function `{function_name}` appears to be a duration \
                         but has no type annotation",
                        param.name
                    ),
                ),
                _ => unreachable!(),
            };

            let suggestion = format!(
                "Change parameter `{}` type from `{current_type}` to `duration`:\n  {}{}:  \
                 duration",
                param.name,
                param.name,
                if is_optional { "?" } else { "" }
            );

            RuleViolation::new_dynamic("prefer_duration_type", message, function_span)
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
        "prefer_duration_type",
        RuleCategory::TypeSafety,
        Severity::Warning,
        "Use Nushell's duration type instead of string for parameters with duration-related names",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
