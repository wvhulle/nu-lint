use nu_protocol::{
    Span, VarId,
    ast::{Block, Expr, Expression, FindMapResult, Traverse},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt, span::SpanExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

/// Semantic fix data: stores the parameter name, span, and whether it's
/// optional
pub struct FixData {
    param_name: String,
    replace_span: Span,
    is_optional: bool,
}

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

fn detect_parameter(
    param: &nu_protocol::PositionalArg,
    param_var_id: VarId,
    block: &Block,
    function_name: &str,
    signature_span: Span,
    is_optional: bool,
    context: &LintContext,
) -> Option<(Detection, FixData)> {
    use nu_protocol::SyntaxShape::{Any, String as StringShape};

    if !matches!(param.shape, StringShape | Any) {
        return None;
    }

    if !parameter_used_as_path(block, param_var_id, &param.name, context) {
        return None;
    }

    let var = context.working_set.get_variable(param_var_id);
    let param_span = var.declaration_span;

    let message = format!(
        "Parameter `{}` in `{function_name}` used as path but typed as {}",
        param.name,
        match param.shape {
            StringShape => "string",
            Any => "any",
            _ => "unknown",
        }
    );

    let optional_marker = if is_optional { "?" } else { "" };

    let param_in_sig_span = signature_span.find_substring_span(&param.name, context);
    let param_end = find_param_type_end(param_in_sig_span, signature_span, context);
    let replace_span = Span::new(param_in_sig_span.start, param_end);

    let violation = Detection::from_global_span(message, param_span)
        .with_primary_label("used as path")
        .with_help(format!("Use `{}{optional_marker}: path`", param.name));

    let fix_data = FixData {
        param_name: param.name.clone(),
        replace_span,
        is_optional,
    };

    Some((violation, fix_data))
}

fn find_param_type_end(param_start: Span, signature_span: Span, context: &LintContext) -> usize {
    let sig_text = context.get_span_text(signature_span);
    let param_offset = param_start.start - signature_span.start;

    let after_param = &sig_text[param_offset..];

    for (i, c) in after_param.char_indices() {
        if c == ',' || c == ']' || c == '#' {
            return param_start.start + i;
        }
    }
    signature_span.end
}

fn detect_function_parameters(
    block: &Block,
    function_name: &str,
    signature_span: Span,
    context: &LintContext,
) -> Vec<(Detection, FixData)> {
    let check_param = |param: &nu_protocol::PositionalArg, is_optional: bool| {
        param.var_id.and_then(|var_id| {
            detect_parameter(
                param,
                var_id,
                block,
                function_name,
                signature_span,
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

struct PreferPathType;

impl DetectFix for PreferPathType {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "prefer_path_type"
    }

    fn explanation(&self) -> &'static str {
        "Use Nushell's path type instead of string for parameters with 'path' in the name"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/types_of_data.html#paths")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| {
            let Expr::Call(call) = &expr.expr else {
                return vec![];
            };

            let decl = ctx.working_set.get_decl(call.decl_id);
            if !matches!(decl.name(), "def" | "export def") {
                return vec![];
            }

            let Some(def) = call.extract_function_definition(ctx) else {
                return vec![];
            };

            let Some(sig_arg) = call.get_positional_arg(1) else {
                return vec![];
            };

            let block = ctx.working_set.get_block(def.body);
            detect_function_parameters(block, &def.name, sig_arg.span, ctx)
        })
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let optional_marker = if fix_data.is_optional { "?" } else { "" };
        let new_param_text = format!("{}{optional_marker}: path", fix_data.param_name);
        Some(Fix::with_explanation(
            format!("Change `{}` type to `path`", fix_data.param_name),
            vec![Replacement::new(fix_data.replace_span, new_param_text)],
        ))
    }
}

pub static RULE: &dyn Rule = &PreferPathType;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
