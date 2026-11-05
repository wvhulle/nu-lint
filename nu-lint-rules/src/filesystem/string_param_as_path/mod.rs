use std::ops::ControlFlow;

use nu_lint_analysis::{
    LintLevel,
    ast::{call::CallExt, declaration::param_type_annotation_span},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::{
    Span, VarId,
    ast::{Block, Expr, Traverse},
};

/// Semantic fix data: stores the parameter name, span, and whether it's
/// optional
pub struct FixData {
    parameter_name_span: Span,
    replace_span: Span,
}

fn parameter_used_as_path(block: &Block, var_id: VarId, context: &LintContext) -> bool {
    block
        .find_map(context.working_set, &|expr| {
            if let Expr::Call(call) = &expr.expr
                && call.is_filesystem_command(context)
                && call.uses_variable(var_id)
            {
                return ControlFlow::Break(Some(()));
            }
            ControlFlow::Continue(())
        })
        .is_some()
}

fn detect_parameter(
    param: &nu_protocol::PositionalArg,
    param_var_id: VarId,
    block: &Block,
    function_name: &str,
    signature_span: Span,
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

    let replace_span = param_type_annotation_span(param, signature_span, context)
        .unwrap_or_else(|| Span::new(param_span.end, param_span.end));

    let violation =
        Detection::from_global_span(message, param_span).with_primary_label("used as path");

    let fix_data = FixData {
        parameter_name_span: param_span,
        replace_span,
    };

    Some((violation, fix_data))
}

fn detect_function_parameters(
    block: &Block,
    function_name: &str,
    signature_span: Span,
    context: &LintContext,
) -> Vec<(Detection, FixData)> {
    let check_param = |param: &nu_protocol::PositionalArg| {
        param.var_id.and_then(|var_id| {
            detect_parameter(param, var_id, block, function_name, signature_span, context)
        })
    };

    block
        .signature
        .required_positional
        .iter()
        .filter_map(&check_param)
        .chain(
            block
                .signature
                .optional_positional
                .iter()
                .filter_map(&check_param),
        )
        .chain(
            block
                .signature
                .rest_positional
                .iter()
                .filter_map(&check_param),
        )
        .collect()
}

struct StringAsPath;

impl DetectFix for StringAsPath {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "string_param_as_path"
    }

    fn short_description(&self) -> &'static str {
        "Parameter typed as string but used as filesystem path"
    }

    fn source_link(&self) -> Option<&'static str> {
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

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let param_name = context.span_text(fix_data.parameter_name_span);
        Some(Fix {
            explanation: format!("Change `{param_name}` type to `path`").into(),
            replacements: vec![Replacement::new(
                fix_data.replace_span,
                ": path".to_string(),
            )],
        })
    }
}

pub static RULE: &dyn Rule = &StringAsPath;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
