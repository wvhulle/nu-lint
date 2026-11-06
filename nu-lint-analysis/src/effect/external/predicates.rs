use nu_protocol::ast::ExternalArgument;

use super::has_external_recursive_flag;
use crate::{
    ast::external::literal_content,
    effect::{is_dangerous_path, matches_long_flag, matches_short_flag},
};

pub type Predicate = fn(&[ExternalArgument]) -> bool;

pub const fn always(_args: &[ExternalArgument]) -> bool {
    true
}

#[must_use]
pub fn literal_argument(arg: &ExternalArgument) -> Option<&str> {
    match arg {
        ExternalArgument::Regular(expr) | ExternalArgument::Spread(expr) => literal_content(expr),
    }
}

pub fn has_flag(args: &[ExternalArgument], patterns: &[&str]) -> bool {
    let matches_pattern = |arg_text: &str, pattern: &str| match pattern.strip_prefix("--") {
        Some(_) => matches_long_flag(arg_text, pattern),
        None => pattern
            .strip_prefix('-')
            .filter(|rest| rest.len() == 1)
            .and_then(|rest| rest.chars().next())
            .is_some_and(|flag_char| {
                matches_long_flag(arg_text, pattern) || matches_short_flag(arg_text, flag_char)
            }),
    };

    args.iter().filter_map(literal_argument).any(|arg_text| {
        patterns
            .iter()
            .any(|pattern| matches_pattern(arg_text, pattern))
    })
}

#[must_use]
pub fn get_subcommand(args: &[ExternalArgument]) -> Option<&str> {
    args.first().and_then(literal_argument)
}

pub fn has_dangerous_path_arg(args: &[ExternalArgument]) -> bool {
    args.iter()
        .filter_map(literal_argument)
        .any(is_dangerous_path)
}

pub fn rm_is_dangerous(args: &[ExternalArgument]) -> bool {
    has_dangerous_path_arg(args) || has_external_recursive_flag(args)
}
