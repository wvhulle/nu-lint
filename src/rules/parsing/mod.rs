use nu_protocol::ast::{Call, Expr};

use crate::{
    ast::{
        call::CallExt,
        regex::{contains_regex_special_chars, escape_regex},
        string::StringFormat,
    },
    context::LintContext,
};

pub mod lines_each_to_parse;
pub mod lines_instead_of_split;
pub mod simplify_regex;
pub mod split_first_to_parse;
pub mod split_row_get_inline;
pub mod split_row_get_multistatement;
pub mod split_row_space_to_split_words;

pub fn is_split_row_call(call: &Call, context: &LintContext) -> bool {
    call.is_call_to_command("split row", context)
}

pub fn is_split_call(call: &Call, context: &LintContext) -> bool {
    matches!(call.get_call_name(context).as_str(), "split row" | "split")
}

pub fn is_indexed_access_call(call: &Call, context: &LintContext) -> bool {
    matches!(call.get_call_name(context).as_str(), "get" | "skip")
}

pub fn extract_index_from_call(call: &Call, context: &LintContext) -> Option<usize> {
    call.get_first_positional_arg()
        .and_then(|arg| context.expr_text(arg).parse().ok())
}

pub fn extract_delimiter_from_split_call(call: &Call, context: &LintContext) -> Option<String> {
    if !is_split_call(call, context) {
        return None;
    }
    let arg = call.get_first_positional_arg()?;
    match &arg.expr {
        Expr::String(s) | Expr::RawString(s) => Some(s.clone()),
        _ => StringFormat::from_expression(arg, context).map(|fmt| fmt.content().to_string()),
    }
}

pub fn needs_regex_for_delimiter(delimiter: &str) -> bool {
    contains_regex_special_chars(delimiter)
}

pub fn generate_parse_pattern(delimiter: &str, num_fields: usize) -> (String, bool) {
    let needs_regex = needs_regex_for_delimiter(delimiter);

    if needs_regex {
        let escaped = escape_regex(delimiter);
        let pattern = (0..num_fields)
            .map(|i| format!("(?P<field{i}>.*)"))
            .collect::<Vec<_>>()
            .join(&escaped);
        (pattern, true)
    } else {
        let pattern = (0..num_fields)
            .map(|i| format!("{{field{i}}}"))
            .collect::<Vec<_>>()
            .join(delimiter);
        (pattern, false)
    }
}

pub fn generate_parse_replacement(delimiter: &str, indexed_fields: &[usize]) -> String {
    let max_field = indexed_fields.iter().copied().max().unwrap_or(0);
    let num_fields = max_field + 2;
    let (pattern, needs_regex) = generate_parse_pattern(delimiter, num_fields);

    if needs_regex {
        format!("parse --regex '{pattern}'")
    } else {
        format!("parse \"{pattern}\"")
    }
}
