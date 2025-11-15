use nu_protocol::{
    BlockId, Span, Type,
    ast::{Call, Expr},
};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt, span::SpanExt, syntax_shape::SyntaxShapeExt},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement, Violation},
};

fn has_explicit_type_annotation(signature_span: Option<Span>, ctx: &LintContext) -> bool {
    signature_span.is_some_and(|span| span.text(ctx).contains("->"))
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

fn create_violations_for_untyped_io(
    func_name: &str,
    name_span: Span,
    uses_in: bool,
    needs_input_type: bool,
    needs_output_type: bool,
    fix: &Fix,
) -> Vec<Violation> {
    if !needs_input_type && !needs_output_type {
        return vec![];
    }

    let (message, suggestion) = match (needs_input_type, needs_output_type) {
        (true, true) => (
            format!(
                "Custom command '{func_name}' uses pipeline input ($in) and produces output but \
                 lacks type annotations"
            ),
            "Add pipeline input and output type annotations (e.g., `: string -> list<int>` or `: \
             any -> table`)",
        ),
        (true, false) => (
            format!(
                "Custom command '{func_name}' uses pipeline input ($in) but lacks input type \
                 annotation"
            ),
            "Add pipeline input type annotation (e.g., `: string -> any` or `: list<int> -> any`)",
        ),
        (false, true) => (
            format!(
                "Custom command '{func_name}' produces output but lacks output type annotation"
            ),
            if uses_in {
                "Add pipeline output type annotation (e.g., `: any -> string` or `: list<int> -> \
                 table`)"
            } else {
                "Add pipeline output type annotation (e.g., `: nothing -> string` or `: nothing -> \
                 list<int>`)"
            },
        ),
        (false, false) => unreachable!(),
    };

    vec![
        Violation::new_dynamic("typed_pipeline_io", message, name_span)
            .with_suggestion_static(suggestion)
            .with_fix(fix.clone()),
    ]
}

fn generate_typed_signature(
    signature: &nu_protocol::Signature,
    ctx: &LintContext,
    block_id: BlockId,
    uses_in: bool,
    needs_input_type: bool,
    needs_output_type: bool,
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
    let params_text = if has_no_params {
        String::new()
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

    match (needs_input_type, needs_output_type) {
        (false, false) => format!("[{params_text}]"),
        _ => format!("[{params_text}]: {input_type} -> {output_type}"),
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
    let needs_input_type = uses_in && is_untyped(signature, sig_span, ctx, |(input, _)| input);
    let needs_output_type =
        produces_out && is_untyped(signature, sig_span, ctx, |(_, output)| output);

    if !needs_input_type && !needs_output_type {
        return vec![];
    }

    let Some(sig_span) = sig_span else {
        return vec![];
    };

    let new_signature = generate_typed_signature(
        signature,
        ctx,
        block_id,
        uses_in,
        needs_input_type,
        needs_output_type,
    );

    let fix = Fix::new_dynamic(
        format!("Add type annotations: {new_signature}"),
        vec![Replacement::new_dynamic(sig_span, new_signature)],
    );

    create_violations_for_untyped_io(
        &func_name,
        name_span,
        uses_in,
        needs_input_type,
        needs_output_type,
        &fix,
    )
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| match &expr.expr {
        Expr::Call(call) => check_def_call(call, ctx),
        _ => vec![],
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "typed_pipeline_io",
        LintLevel::Warn,
        "Custom commands that use pipeline input or produce output should have type annotations",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
