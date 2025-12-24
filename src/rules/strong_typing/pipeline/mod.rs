use nu_protocol::{
    BlockId, Span, Type,
    ast::{Block, Call, Expr},
};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt, span::SpanExt, syntax_shape::SyntaxShapeExt},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement, Violation},
};

fn find_return_span(block: &Block) -> Option<Span> {
    block
        .pipelines
        .last()
        .and_then(|p| p.elements.last())
        .map(|e| e.expr.span)
}

fn has_explicit_type_annotation(signature_span: Option<Span>, ctx: &LintContext) -> bool {
    signature_span.is_some_and(|span| span.source_code(ctx).contains("->"))
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
    fix: &Fix,
) -> Vec<Violation> {
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

    let mut violation = Violation::new(message, name_span)
        .with_primary_label(label)
        .with_fix(fix.clone());

    if needs_input_type && let Some(span) = in_usage_span {
        violation = violation.with_extra_label("$in used here", span);
    }

    if needs_output_type && let Some(span) = return_span {
        violation = violation.with_extra_label("returned here", span);
    }

    vec![violation]
}

fn generate_typed_signature(
    signature: &nu_protocol::Signature,
    ctx: &LintContext,
    block_id: BlockId,
    uses_in: bool,
    needs_input_type: bool,
    needs_output_type: bool,
    original_sig_span: Span,
) -> String {
    let has_no_params = signature.required_positional.is_empty()
        && signature.optional_positional.is_empty()
        && signature.rest_positional.is_none()
        && signature.named.is_empty();
    log::debug!(
        "Generating typed signature for block {block_id:?}: has_no_params={has_no_params}, \
         uses_in={uses_in}, needs_input_type={needs_input_type}, \
         needs_output_type={needs_output_type}"
    );

    let original_sig_text = original_sig_span.source_code(ctx);
    let is_multiline = has_multiline_parameters(original_sig_text);

    let params_text = if has_no_params {
        String::new()
    } else if is_multiline {
        // Preserve multiline formatting - extract parameters from original text
        extract_parameters_from_original(original_sig_text)
    } else {
        extract_parameters_text(signature)
    };

    let block = ctx.working_set.get_block(block_id);

    let input_type = if uses_in || needs_input_type {
        block.infer_input_type(ctx)
    } else {
        Type::Nothing
    };

    let output_type = if needs_output_type {
        block.infer_output_type(ctx)
    } else {
        Type::Any
    };

    if needs_input_type || needs_output_type {
        format!("[{params_text}]: {input_type} -> {output_type}")
    } else {
        format!("[{params_text}]")
    }
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

fn check_def_call(call: &Call, ctx: &LintContext) -> Vec<Violation> {
    let Some((block_id, func_name)) = call.extract_function_definition(ctx) else {
        return vec![];
    };
    log::debug!("Checking function definition for typed_pipeline_io: {func_name}");

    let Some((_, name_span)) = call.extract_declaration_name(ctx) else {
        return vec![];
    };

    let block = ctx.working_set.get_block(block_id);
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

    let new_signature = generate_typed_signature(
        signature,
        ctx,
        block_id,
        uses_in,
        needs_input_type,
        needs_output_type,
        sig_span,
    );

    let fix = Fix::with_explanation(
        format!("Add type annotations: {new_signature}"),
        vec![Replacement::new(sig_span, new_signature)],
    );

    create_violations_for_untyped_io(
        &func_name,
        name_span,
        uses_in,
        needs_input_type,
        needs_output_type,
        in_usage_span,
        return_span,
        &fix,
    )
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| match &expr.expr {
        Expr::Call(call) => check_def_call(call, ctx),
        _ => vec![],
    })
}

pub const RULE: Rule = Rule::new(
    "typed_pipeline_io",
    "Custom commands that use pipeline input or produce output should have type annotations",
    check,
    LintLevel::Warning,
)
.with_auto_fix()
.with_doc_url("https://www.nushell.sh/book/custom_commands.html#input-output-types");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
