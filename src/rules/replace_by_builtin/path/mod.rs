use nu_protocol::{
    VarId,
    ast::{Block, Expr, Expression, FindMapResult, Traverse},
};

use crate::{
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::Rule,
    violation::Violation,
};

const PATH_KEYWORDS: &[&str] = &["path", "file", "dir", "directory", "folder", "location"];

fn is_likely_filesystem_param(param_name: &str) -> bool {
    let lower_name = param_name.to_lowercase();
    lower_name
        .split(|c: char| c == '_' || c == '-' || !c.is_alphanumeric())
        .any(|part| {
            PATH_KEYWORDS
                .iter()
                .any(|&kw| part == kw || part.ends_with(kw))
        })
}

fn check_nu_builtin_usage(expr: &Expression, var_id: VarId, context: &LintContext) -> bool {
    if let Expr::Call(call) = &expr.expr {
        call.is_filesystem_command(context) && call.uses_variable(var_id)
    } else {
        false
    }
}

fn check_external_command_usage(
    expr: &Expression,
    var_id: VarId,
    param_name: &str,
    context: &LintContext,
) -> bool {
    expr.is_external_filesystem_command(context)
        && expr.external_call_contains_variable(var_id)
        && is_likely_filesystem_param(param_name)
}

fn parameter_used_as_path(
    block: &Block,
    var_id: VarId,
    param_name: &str,
    context: &LintContext,
) -> bool {
    block
        .find_map(context.working_set, &|expr| {
            if check_nu_builtin_usage(expr, var_id, context)
                || check_external_command_usage(expr, var_id, param_name, context)
            {
                FindMapResult::Found(())
            } else {
                FindMapResult::Continue
            }
        })
        .is_some()
}
fn check_parameter(
    param: &nu_protocol::PositionalArg,
    param_var_id: VarId,
    block: &Block,
    function_name: &str,
    function_span: nu_protocol::Span,
    is_optional: bool,
    context: &LintContext,
) -> Option<Violation> {
    use nu_protocol::SyntaxShape::{Any, String as StringShape};

    if !matches!(param.shape, StringShape | Any) {
        return None;
    }

    if !parameter_used_as_path(block, param_var_id, &param.name, context) {
        return None;
    }

    let (current_type, message) = match param.shape {
        StringShape => (
            "string",
            format!(
                "Parameter `{}` in function `{function_name}` is used as a path but has `string` \
                 type",
                param.name
            ),
        ),
        Any => (
            "no type annotation",
            format!(
                "Parameter `{}` in function `{function_name}` is used as a path but has no type \
                 annotation",
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

    Some(
        Violation::new_dynamic("prefer_path_type", message, function_span)
            .with_suggestion_dynamic(suggestion),
    )
}

fn check_function_parameters(
    block: &Block,
    function_name: &str,
    function_span: nu_protocol::Span,
    context: &LintContext,
) -> Vec<Violation> {
    let check_param = |param: &nu_protocol::PositionalArg, is_optional: bool| {
        param.var_id.and_then(|var_id| {
            check_parameter(
                param,
                var_id,
                block,
                function_name,
                function_span,
                is_optional,
                context,
            )
        })
    };

    block
        .signature
        .required_positional
        .iter()
        .filter_map(|param| check_param(param, false))
        .chain(
            block
                .signature
                .optional_positional
                .iter()
                .filter_map(|param| check_param(param, true)),
        )
        .chain(
            block
                .signature
                .rest_positional
                .iter()
                .filter_map(|param| check_param(param, false)),
        )
        .collect()
}

fn check(context: &LintContext) -> Vec<Violation> {
    context
        .collect_function_definitions()
        .iter()
        .flat_map(|(block_id, function_name)| {
            let block = context.working_set.get_block(*block_id);
            let function_span = context.find_declaration_span(function_name);
            check_function_parameters(block, function_name, function_span, context)
        })
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_path_type",
        "Use Nushell's path type instead of string for parameters with 'path' in the name",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
