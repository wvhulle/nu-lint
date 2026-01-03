use nu_protocol::{
    Span, VarId,
    ast::{Block, Expr, FindMapResult, Traverse},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, span::SpanExt},
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

fn parameter_used_as_path(block: &Block, var_id: VarId, context: &LintContext) -> bool {
    block
        .find_map(context.working_set, &|expr| {
            if let Expr::Call(call) = &expr.expr
                && call.is_filesystem_command(context)
                && call.uses_variable(var_id)
            {
                return FindMapResult::Found(());
            }
            FindMapResult::Continue
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

    if !parameter_used_as_path(block, param_var_id, context) {
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

    let param_in_sig_span = signature_span.find_substring_span(&param.name, context);
    let param_end = find_param_type_end(param_in_sig_span, signature_span, context);
    let replace_span = Span::new(param_in_sig_span.start, param_end);

    let violation =
        Detection::from_global_span(message, param_span).with_primary_label("used as path");

    let fix_data = FixData {
        param_name: param.name.clone(),
        replace_span,
        is_optional,
    };

    Some((violation, fix_data))
}

fn find_param_type_end(param_start: Span, signature_span: Span, context: &LintContext) -> usize {
    let sig_text = context.plain_text(signature_span);
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

struct StringAsPath;

impl DetectFix for StringAsPath {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "string_param_as_path"
    }

    fn explanation(&self) -> &'static str {
        "Parameter typed as string but used as filesystem path"
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

            let Some(def) = call.custom_command_def(ctx) else {
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

pub static RULE: &dyn Rule = &StringAsPath;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
