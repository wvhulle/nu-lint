use nu_protocol::{
    BlockId, Span, Type,
    ast::{Block, Call, Expr},
};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt, syntax_shape::SyntaxShapeExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

/// Semantic fix data: stores information needed to regenerate the fix
pub struct FixData {
    sig_span: Span,
    body_block_id: BlockId,
    uses_in: bool,
    needs_input_type: bool,
    needs_output_type: bool,
}

fn find_return_span(block: &Block) -> Option<Span> {
    block
        .pipelines
        .last()
        .and_then(|p| p.elements.last())
        .map(|e| e.expr.span)
}

fn has_explicit_type_annotation(signature_span: Option<Span>, ctx: &LintContext) -> bool {
    signature_span.is_some_and(|span| ctx.get_span_text(span).contains("->"))
}

fn is_untyped<F>(
    signature: &nu_protocol::Signature,
    signature_span: Option<Span>,
    ctx: &LintContext,
    selector: F,
) -> bool
where
    F: Fn(&(nu_protocol::Type, nu_protocol::Type)) -> &nu_protocol::Type,
{
    !has_explicit_type_annotation(signature_span, ctx)
        && signature
            .input_output_types
            .iter()
            .all(|types| matches!(selector(types), nu_protocol::Type::Any))
}

fn find_signature_span(call: &Call, _ctx: &LintContext) -> Option<Span> {
    let sig_arg = call.get_positional_arg(1)?;
    Some(sig_arg.span)
}

#[allow(
    clippy::too_many_arguments,
    reason = "Grouping related parameters for violation creation"
)]
fn create_violations_for_untyped_io(
    func_name: &str,
    name_span: Span,
    _uses_in: bool,
    needs_input_type: bool,
    needs_output_type: bool,
    in_usage_span: Option<Span>,
    return_span: Option<Span>,
    fix_data: FixData,
) -> Vec<(Detection, FixData)> {
    if !needs_input_type && !needs_output_type {
        return vec![];
    }

    let (message, label) = match (needs_input_type, needs_output_type) {
        (true, true) => (
            format!("'{func_name}' missing input/output types"),
            "add type annotation",
        ),
        (true, false) => (
            format!("'{func_name}' missing input type"),
            "add input type",
        ),
        (false, true) => (
            format!("'{func_name}' missing output type"),
            "add output type",
        ),
        (false, false) => unreachable!(),
    };

    let mut violation = Detection::from_global_span(message, name_span).with_primary_label(label);

    if needs_input_type && let Some(span) = in_usage_span {
        violation = violation.with_extra_label("$in used here", span);
    }

    if needs_output_type && let Some(span) = return_span {
        violation = violation.with_extra_label("returned here", span);
    }

    vec![(violation, fix_data)]
}

fn extract_parameters_from_original(sig_text: &str) -> String {
    if let Some(start) = sig_text.find('[')
        && let Some(end) = sig_text.rfind(']')
        && start < end
    {
        sig_text[start + 1..end].to_string()
    } else {
        sig_text.to_string()
    }
}

fn has_multiline_parameters(sig_text: &str) -> bool {
    if let Some(start) = sig_text.find('[')
        && let Some(end) = sig_text.rfind(']')
        && start < end
    {
        sig_text[start + 1..end].contains('\n')
    } else {
        false
    }
}

fn extract_parameters_text(signature: &nu_protocol::Signature) -> String {
    let required = signature
        .required_positional
        .iter()
        .map(|param| format_positional(&param.name, &param.shape, false, false));

    let optional = signature
        .optional_positional
        .iter()
        .map(|param| format_positional(&param.name, &param.shape, true, false));

    let rest = signature
        .rest_positional
        .iter()
        .map(|rest| format_positional(&rest.name, &rest.shape, false, true));

    let flags = signature
        .named
        .iter()
        .filter(|flag| flag.long != "help")
        .map(|flag| match (&flag.short, &flag.arg) {
            (Some(short), Some(arg_shape)) => {
                format!(
                    "--{} (-{}): {}",
                    flag.long,
                    short,
                    shape_to_string(arg_shape)
                )
            }
            (Some(short), None) => format!("--{} (-{})", flag.long, short),
            (None, Some(arg_shape)) => {
                format!("--{}: {}", flag.long, shape_to_string(arg_shape))
            }
            (None, None) => format!("--{}", flag.long),
        });

    required
        .chain(optional)
        .chain(rest)
        .chain(flags)
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_positional(
    name: &str,
    shape: &nu_protocol::SyntaxShape,
    optional: bool,
    rest: bool,
) -> String {
    let prefix = if rest { "..." } else { "" };
    let suffix = if optional { "?" } else { "" };

    match shape {
        nu_protocol::SyntaxShape::Any => format!("{prefix}{name}{suffix}"),
        _ => format!("{prefix}{name}{suffix}: {}", shape_to_string(shape)),
    }
}

fn shape_to_string(shape: &nu_protocol::SyntaxShape) -> String {
    shape.to_type_string()
}

fn detect_def_call(call: &Call, ctx: &LintContext) -> Vec<(Detection, FixData)> {
    let Some(def) = call.custom_command_def(ctx) else {
        return vec![];
    };
    log::debug!(
        "Checking function definition for typed_pipeline_io: {}",
        def.name
    );

    let block = ctx.working_set.get_block(def.body);
    let signature = &block.signature;
    let sig_span = find_signature_span(call, ctx);

    let uses_in = block.uses_pipeline_input(ctx);
    let produces_out = block.produces_output();

    let inferred_output = block.infer_output_type(ctx);
    let output_is_nothing = matches!(inferred_output, Type::Nothing);

    let needs_input_type = uses_in && is_untyped(signature, sig_span, ctx, |(input, _)| input);
    let needs_output_type = produces_out
        && !output_is_nothing
        && is_untyped(signature, sig_span, ctx, |(_, output)| output);

    if !needs_input_type && !needs_output_type {
        return vec![];
    }

    let Some(sig_span) = sig_span else {
        return vec![];
    };

    let in_usage_span = if needs_input_type {
        block.find_dollar_in_usage()
    } else {
        None
    };

    let return_span = if needs_output_type {
        find_return_span(block)
    } else {
        None
    };

    let fix_data = FixData {
        sig_span,
        body_block_id: def.body,
        uses_in,
        needs_input_type,
        needs_output_type,
    };

    create_violations_for_untyped_io(
        &def.name,
        def.name_span,
        uses_in,
        needs_input_type,
        needs_output_type,
        in_usage_span,
        return_span,
        fix_data,
    )
}

struct TypedPipelineIo;

impl DetectFix for TypedPipelineIo {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "typed_pipeline_io"
    }

    fn explanation(&self) -> &'static str {
        "Custom commands that use pipeline input or produce output should have pipeline type \
         annotations"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/custom_commands.html#input-output-types")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| match &expr.expr {
            Expr::Call(call) => detect_def_call(call, ctx),
            _ => vec![],
        })
    }

    fn fix(&self, ctx: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let block = ctx.working_set.get_block(fix_data.body_block_id);
        let has_no_params = block.signature.required_positional.is_empty()
            && block.signature.optional_positional.is_empty()
            && block.signature.rest_positional.is_none()
            && block.signature.named.is_empty();

        let original_sig_text = ctx.get_span_text(fix_data.sig_span);
        let is_multiline = has_multiline_parameters(original_sig_text);

        let params_text = if has_no_params {
            String::new()
        } else if is_multiline {
            // Preserve multiline formatting - extract parameters from original text
            extract_parameters_from_original(original_sig_text)
        } else {
            extract_parameters_text(&block.signature)
        };

        let block = ctx.working_set.get_block(fix_data.body_block_id);

        let input_type = if fix_data.uses_in || fix_data.needs_input_type {
            block.infer_input_type(ctx)
        } else {
            Type::Nothing
        };

        let output_type = if fix_data.needs_output_type {
            block.infer_output_type(ctx)
        } else {
            Type::Any
        };

        let new_signature = {
            if fix_data.needs_input_type || fix_data.needs_output_type {
                format!("[{params_text}]: {input_type} -> {output_type}")
            } else {
                format!("[{params_text}]")
            }
        };

        Some(Fix::with_explanation(
            format!("Add type annotations: {new_signature}"),
            vec![Replacement::new(fix_data.sig_span, new_signature)],
        ))
    }
}

pub static RULE: &dyn Rule = &TypedPipelineIo;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
