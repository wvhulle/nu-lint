use nu_protocol::ast::ExternalArgument;

use super::has_external_recursive_flag;
use crate::{
    context::LintContext,
    effect::{is_dangerous_path, is_unvalidated_variable, matches_long_flag, matches_short_flag},
};

pub type Predicate = fn(&LintContext<'_>, &[ExternalArgument]) -> bool;

pub const fn always(_context: &LintContext, _args: &[ExternalArgument]) -> bool {
    true
}

pub fn extract_external_arg_text<'a>(arg: &ExternalArgument, context: &'a LintContext) -> &'a str {
    match arg {
        ExternalArgument::Regular(expr) | ExternalArgument::Spread(expr) => context.expr_text(expr),
    }
}

pub fn has_flag(args: &[ExternalArgument], context: &LintContext, patterns: &[&str]) -> bool {
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

    args.iter()
        .map(|arg| extract_external_arg_text(arg, context))
        .any(|arg_text| {
            patterns
                .iter()
                .any(|pattern| matches_pattern(arg_text, pattern))
        })
}

pub fn get_subcommand<'a>(args: &[ExternalArgument], context: &'a LintContext) -> &'a str {
    args.first()
        .map_or("", |arg| extract_external_arg_text(arg, context))
}

pub fn has_dangerous_path_arg(context: &LintContext, args: &[ExternalArgument]) -> bool {
    args.iter()
        .map(|arg| extract_external_arg_text(arg, context))
        .any(|path| is_dangerous_path(path) || is_unvalidated_variable(path))
}

pub fn rm_is_dangerous(context: &LintContext, args: &[ExternalArgument]) -> bool {
    has_dangerous_path_arg(context, args) || has_external_recursive_flag(args, context)
}
