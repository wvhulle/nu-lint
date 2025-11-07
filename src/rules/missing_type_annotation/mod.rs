use nu_protocol::ast::{Argument, Call, Expr, Expression, Operator, Pipeline};

use crate::{
    ast::{
        builtin_command::CommandExt, call::CallExt, expression::ExpressionExt,
        syntax_shape::SyntaxShapeExt,
    },
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{Fix, Replacement, RuleViolation, Severity},
};

fn is_reference_to_param(expr: &Expression, param_name: &str, ctx: &LintContext) -> bool {
    match &expr.expr {
        Expr::Var(var_id) => {
            let var = ctx.working_set.get_variable(*var_id);
            let var_span_text = &ctx.source[var.declaration_span.start..var.declaration_span.end];
            let normalized = var_span_text
                .trim_end_matches('?')
                .trim_start_matches("...");
            normalized == param_name
        }
        Expr::FullCellPath(cell_path) => is_reference_to_param(&cell_path.head, param_name, ctx),
        _ => false,
    }
}

fn infer_type_from_pipeline(
    param_name: &str,
    pipeline: &Pipeline,
    ctx: &LintContext,
) -> Option<&'static str> {
    pipeline
        .elements
        .windows(2)
        .find_map(|window| match &window[1].expr.expr {
            Expr::Call(call) if is_reference_to_param(&window[0].expr, param_name, ctx) => {
                call.get_call_name(ctx).as_str().output_type()
            }
            _ => None,
        })
}

fn infer_type_from_block(
    param_name: &str,
    block_id: nu_protocol::BlockId,
    ctx: &LintContext,
) -> Option<&'static str> {
    let block = ctx.working_set.get_block(block_id);

    block.pipelines.iter().find_map(|pipeline| {
        infer_type_from_pipeline(param_name, pipeline, ctx).or_else(|| {
            pipeline
                .elements
                .iter()
                .find_map(|element| infer_type_from_expr(&element.expr, param_name, ctx))
        })
    })
}

fn infer_type_from_expr(
    expr: &Expression,
    param_name: &str,
    ctx: &LintContext,
) -> Option<&'static str> {
    match &expr.expr {
        Expr::Call(call) if is_reference_to_param(expr, param_name, ctx) => {
            call.get_call_name(ctx).as_str().output_type()
        }
        Expr::Call(call) => call.arguments.iter().find_map(|arg| match arg {
            Argument::Positional(arg_expr) | Argument::Unknown(arg_expr) => {
                infer_type_from_expr(arg_expr, param_name, ctx)
            }
            _ => None,
        }),
        Expr::FullCellPath(cell_path)
            if is_reference_to_param(&cell_path.head, param_name, ctx)
                && !cell_path.tail.is_empty() =>
        {
            Some("record")
        }
        Expr::FullCellPath(cell_path) => infer_type_from_expr(&cell_path.head, param_name, ctx),
        Expr::BinaryOp(left, op_expr, right)
            if matches!(&op_expr.expr, Expr::Operator(op) if matches!(op, Operator::Math(_) | Operator::Comparison(_)))
                && (is_reference_to_param(left, param_name, ctx)
                    || is_reference_to_param(right, param_name, ctx)) =>
        {
            Some("int")
        }
        Expr::BinaryOp(left, _, right) => infer_type_from_expr(left, param_name, ctx)
            .or_else(|| infer_type_from_expr(right, param_name, ctx)),
        Expr::Subexpression(block_id) | Expr::Block(block_id) | Expr::Closure(block_id) => {
            infer_type_from_block(param_name, *block_id, ctx)
        }
        Expr::MatchBlock(patterns) => patterns
            .iter()
            .find_map(|(_, expr)| infer_type_from_expr(expr, param_name, ctx)),
        Expr::Collect(_, inner) | Expr::UnaryNot(inner) => {
            infer_type_from_expr(inner, param_name, ctx)
        }
        _ => None,
    }
}

fn infer_param_type(
    param_name: &str,
    body_block_id: nu_protocol::BlockId,
    ctx: &LintContext,
) -> &'static str {
    infer_type_from_block(param_name, body_block_id, ctx).unwrap_or("any")
}

fn get_param_type_str(
    shape: &nu_protocol::SyntaxShape,
    name: &str,
    body_block_id: nu_protocol::BlockId,
    ctx: &LintContext,
) -> String {
    if *shape == nu_protocol::SyntaxShape::Any {
        infer_param_type(name, body_block_id, ctx).to_string()
    } else {
        shape.to_type_string()
    }
}

fn generate_typed_signature(
    signature: &nu_protocol::Signature,
    body_block_id: nu_protocol::BlockId,
    ctx: &LintContext,
) -> String {
    let params = signature
        .required_positional
        .iter()
        .map(|p| {
            format!(
                "{}: {}",
                p.name,
                get_param_type_str(&p.shape, &p.name, body_block_id, ctx)
            )
        })
        .chain(signature.optional_positional.iter().map(|p| {
            format!(
                "{}?: {}",
                p.name,
                get_param_type_str(&p.shape, &p.name, body_block_id, ctx)
            )
        }))
        .chain(signature.rest_positional.iter().map(|p| {
            format!(
                "...{}: {}",
                p.name,
                get_param_type_str(&p.shape, &p.name, body_block_id, ctx)
            )
        }))
        .collect::<Vec<_>>()
        .join(", ");

    format!("[{params}]")
}

fn find_param_span(
    signature_span: nu_protocol::Span,
    param_name: &str,
    ctx: &LintContext,
) -> nu_protocol::Span {
    ctx.working_set
        .get_span_contents(signature_span)
        .windows(param_name.len())
        .position(|window| window == param_name.as_bytes())
        .map_or(signature_span, |offset| {
            nu_protocol::Span::new(
                signature_span.start + offset,
                signature_span.start + offset + param_name.len(),
            )
        })
}

fn check_signature(
    sig: &nu_protocol::Signature,
    signature_span: nu_protocol::Span,
    body_block_id: nu_protocol::BlockId,
    ctx: &LintContext,
) -> Vec<RuleViolation> {
    let params_needing_types: Vec<_> = sig
        .required_positional
        .iter()
        .chain(&sig.optional_positional)
        .chain(sig.rest_positional.iter())
        .filter(|param| param.shape == nu_protocol::SyntaxShape::Any)
        .collect();

    if params_needing_types.is_empty() {
        return vec![];
    }

    let new_sig = generate_typed_signature(sig, body_block_id, ctx);
    let fix = Fix::new_static(
        "Add type annotations to parameters",
        vec![Replacement::new_dynamic(signature_span, new_sig)],
    );

    params_needing_types
        .into_iter()
        .map(|param| {
            let param_span = find_param_span(signature_span, &param.name, ctx);
            RuleViolation::new_dynamic(
                "missing_type_annotation",
                format!("Parameter '{}' is missing type annotation", param.name),
                param_span,
            )
            .with_suggestion_static("Add type annotation like 'param: string' or 'param: int'")
            .with_fix(fix.clone())
        })
        .collect()
}

fn check_def_call(call: &Call, ctx: &LintContext) -> Vec<RuleViolation> {
    let decl = ctx.working_set.get_decl(call.decl_id);

    (decl.name() == "def" || decl.name() == "export def")
        .then(|| {
            call.get_positional_arg(1)
                .zip(call.get_positional_arg(2))
                .and_then(|(sig_arg, body_arg)| {
                    body_arg
                        .extract_block_id()
                        .and_then(|body_block_id| match &sig_arg.expr {
                            Expr::Signature(sig) => {
                                Some(check_signature(sig, sig_arg.span, body_block_id, ctx))
                            }
                            _ => None,
                        })
                })
        })
        .flatten()
        .unwrap_or_default()
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context.collect_rule_violations(|expr, ctx| match &expr.expr {
        Expr::Call(call) => check_def_call(call, ctx),
        _ => vec![],
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "missing_type_annotation",
        RuleCategory::TypeSafety,
        Severity::Warning,
        "Parameters should have type annotations",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
