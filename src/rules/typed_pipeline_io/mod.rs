use nu_protocol::{
    BlockId, Span, VarId,
    ast::{Call, Expr, Expression, PathMember},
};

use crate::{
    ast::{
        block::BlockExt, builtin_command::CommandExt, call::CallExt,
        ext_command::ExternalCommandExt, span::SpanExt, syntax_shape::SyntaxShapeExt,
    },
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{Fix, Replacement, RuleViolation, Severity},
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
) -> Vec<RuleViolation> {
    [
        (
            needs_input_type,
            format!(
                "Custom command '{func_name}' uses pipeline input ($in) but lacks input type \
                 annotation"
            ),
            "Add pipeline input type annotation (e.g., `: string -> any` or `: list<int> -> any`)",
        ),
        (
            needs_output_type,
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
    ]
    .into_iter()
    .filter_map(|(needs, message, suggestion)| {
        needs.then(|| {
            RuleViolation::new_dynamic("typed_pipeline_io", message, name_span)
                .with_suggestion_static(suggestion)
                .with_fix(fix.clone())
        })
    })
    .collect()
}

const fn is_filepath_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Filepath(..) | Expr::GlobPattern(..))
}

fn check_filepath_output(expr: &Expr) -> Option<&'static str> {
    match expr {
        Expr::ExternalCall(head, _) if is_filepath_expr(&head.expr) => Some("path"),
        Expr::Collect(_, inner) if is_filepath_expr(&inner.expr) => Some("path"),
        expr if is_filepath_expr(expr) => Some("path"),
        _ => None,
    }
}

fn infer_output_type(block_id: BlockId, ctx: &LintContext) -> String {
    let block = ctx.working_set.get_block(block_id);
    log::debug!("Inferring output type for block {block_id:?}");

    block
        .pipelines
        .last()
        .and_then(|pipeline| pipeline.elements.last())
        .and_then(|elem| infer_from_expression(&elem.expr, ctx))
        .unwrap_or_else(|| block.output_type().to_string())
}

fn infer_from_expression(expr: &Expression, ctx: &LintContext) -> Option<String> {
    let inner_expr = match &expr.expr {
        Expr::Collect(_, inner) => &inner.expr,
        _ => &expr.expr,
    };

    match inner_expr {
        expr if check_filepath_output(expr).is_some() => {
            check_filepath_output(expr).map(String::from)
        }
        Expr::Subexpression(block_id) | Expr::Block(block_id) => {
            Some(infer_output_type(*block_id, ctx))
        }
        Expr::ExternalCall(call, _) => {
            infer_command_output_type_external(call, ctx).map(String::from)
        }
        Expr::Call(call) => infer_from_call_with_blocks(call, ctx)
            .or_else(|| Some(infer_command_output_type_internal(call, ctx))),
        _ => None,
    }
}

fn infer_from_call_with_blocks(call: &Call, ctx: &LintContext) -> Option<String> {
    let decl = ctx.working_set.get_decl(call.decl_id);
    let cmd_name = decl.name();

    if !matches!(cmd_name, "if" | "match" | "try" | "do") {
        return None;
    }

    let block_types: Vec<String> = call
        .positional_iter()
        .filter_map(|arg| match &arg.expr {
            Expr::Block(block_id) | Expr::Closure(block_id) => {
                Some(infer_output_type(*block_id, ctx))
            }
            _ => None,
        })
        .collect();

    if block_types.is_empty() {
        return None;
    }

    if block_types.iter().all(|t| t == &block_types[0]) {
        return Some(block_types[0].clone());
    }

    if block_types.iter().all(|t| t == "nothing") {
        return Some("nothing".into());
    }

    None
}

fn infer_command_output_type_external(
    external_call: &Expression,
    context: &LintContext,
) -> Option<&'static str> {
    let cmd_name = external_call.span.text(context);
    if cmd_name.is_known_external_no_output_command() {
        Some("nothing")
    } else if cmd_name.is_known_external_output_command() {
        Some("string")
    } else {
        None
    }
}

fn infer_command_output_type_internal(call: &Call, context: &LintContext) -> String {
    let cmd_name = call.get_call_name(context);

    if cmd_name.as_str().is_side_effect_only() {
        return "nothing".into();
    }

    if let Some(output_type) = cmd_name.as_str().output_type() {
        return output_type.to_string();
    }

    let decl = context.working_set.get_decl(call.decl_id);
    let signature = decl.signature();
    let output_type = signature.get_output_type().to_string();
    log::debug!("Command '{cmd_name}' has signature output type: {output_type}");
    output_type
}

fn infer_input_type(block_id: BlockId, ctx: &LintContext) -> String {
    let block = ctx.working_set.get_block(block_id);
    let Some(in_var) = block_id.find_pipeline_input_variable(ctx) else {
        return "any".to_string();
    };

    block
        .pipelines
        .iter()
        .flat_map(|pipeline| &pipeline.elements)
        .find_map(|element| infer_input_from_expression(&element.expr, Some(in_var), ctx))
        .map_or_else(|| "any".to_string(), String::from)
}

fn infer_input_from_expression(
    expr: &Expression,
    in_var: Option<VarId>,
    ctx: &LintContext,
) -> Option<&'static str> {
    let in_var_id = in_var?;

    match &expr.expr {
        Expr::FullCellPath(cell_path) if matches!(&cell_path.head.expr, Expr::Var(var_id) if *var_id == in_var_id) => {
            if !cell_path.tail.is_empty()
                && cell_path
                    .tail
                    .iter()
                    .any(|member| matches!(member, PathMember::String { .. }))
            {
                Some("record")
            } else if !cell_path.tail.is_empty() {
                Some("list")
            } else {
                None
            }
        }
        Expr::Call(call) => call.get_call_name(ctx).as_str().input_type(),
        Expr::Collect(_, inner) | Expr::UnaryNot(inner) => {
            infer_input_from_expression(inner, in_var, ctx)
        }
        Expr::BinaryOp(left, _, right) => infer_input_from_expression(left, in_var, ctx)
            .or_else(|| infer_input_from_expression(right, in_var, ctx)),
        Expr::Subexpression(block_id) | Expr::Block(block_id) | Expr::Closure(block_id) => {
            let block = ctx.working_set.get_block(*block_id);
            block
                .pipelines
                .iter()
                .flat_map(|pipeline| &pipeline.elements)
                .find_map(|element| infer_input_from_expression(&element.expr, in_var, ctx))
        }
        _ => None,
    }
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

    let input_type = if uses_in || needs_input_type {
        infer_input_type(block_id, ctx)
    } else {
        "nothing".to_string()
    };

    let output_type = if needs_output_type {
        infer_output_type(block_id, ctx)
    } else {
        "any".to_string()
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

fn check_def_call(call: &Call, ctx: &LintContext) -> Vec<RuleViolation> {
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

    let uses_in = block_id.uses_pipeline_input(ctx);
    let produces_out = block_id.produces_output(ctx);
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

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context.collect_rule_violations(|expr, ctx| match &expr.expr {
        Expr::Call(call) => check_def_call(call, ctx),
        _ => vec![],
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "typed_pipeline_io",
        RuleCategory::TypeSafety,
        Severity::Warning,
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
