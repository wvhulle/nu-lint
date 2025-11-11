use nu_protocol::{
    Type, VarId,
    ast::{Call, Expr},
};

use crate::{
    ast::{
        call::CallExt, expression::ExpressionExt, pipeline::PipelineExt, span::SpanExt,
        syntax_shape::SyntaxShapeExt,
    },
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{Fix, Replacement, RuleViolation, Severity},
};

fn infer_param_type(
    param_var_id: VarId,
    body_block_id: nu_protocol::BlockId,
    ctx: &LintContext,
) -> Type {
    log::debug!("infer_param_type: param_var_id={param_var_id:?}, body_block_id={body_block_id:?}");
    let block = ctx.working_set.get_block(body_block_id);

    // First try pipeline-based inference
    log::debug!("  Trying pipeline-based inference...");
    let pipeline_type = block
        .pipelines
        .iter()
        .find_map(|pipeline| pipeline.infer_param_type(param_var_id, ctx));

    if let Some(ty) = &pipeline_type {
        log::debug!("  -> Pipeline-based inference found: {ty:?}");
        return ty.clone();
    }

    // Fall back to expression-based inference (handles arguments, closures, binary ops, etc.)
    log::debug!("  Trying expression-based inference...");
    let expr_type = block
        .pipelines
        .iter()
        .flat_map(|pipeline| &pipeline.elements)
        .find_map(|element| {
            let result = element.expr.infer_input_type(Some(param_var_id), ctx);
            log::debug!("    Checked element, result: {result:?}");
            result
        });

    if let Some(ty) = &expr_type {
        log::debug!("  -> Expression-based inference found: {ty:?}");
        return ty.clone();
    }

    log::debug!("  -> No type found, returning Type::Any");
    Type::Any
}

fn get_param_type_str(
    shape: &nu_protocol::SyntaxShape,
    var_id: Option<VarId>,
    body_block_id: nu_protocol::BlockId,
    ctx: &LintContext,
) -> String {
    if *shape == nu_protocol::SyntaxShape::Any {
        log::debug!("Inferring type for parameter {var_id:?} with shape Any");
        var_id.map_or_else(
            || Type::Any.to_string(),
            |var_id| infer_param_type(var_id, body_block_id, ctx).to_string(),
        )
    } else {
        shape.to_type_string()
    }
}

fn generate_typed_signature(
    signature: &nu_protocol::Signature,
    body_block_id: nu_protocol::BlockId,
    ctx: &LintContext,
) -> String {
    log::debug!("Generating typed signature for: {signature:?}");
    let params = signature
        .required_positional
        .iter()
        .map(|p| {
            format!(
                "{}: {}",
                p.name,
                get_param_type_str(&p.shape, p.var_id, body_block_id, ctx)
            )
        })
        .chain(signature.optional_positional.iter().map(|p| {
            format!(
                "{}?: {}",
                p.name,
                get_param_type_str(&p.shape, p.var_id, body_block_id, ctx)
            )
        }))
        .chain(signature.rest_positional.iter().map(|p| {
            format!(
                "...{}: {}",
                p.name,
                get_param_type_str(&p.shape, p.var_id, body_block_id, ctx)
            )
        }))
        .collect::<Vec<_>>()
        .join(", ");

    format!("[{params}]")
}

fn check_signature(
    sig: &nu_protocol::Signature,
    signature_span: nu_protocol::Span,
    body_block_id: nu_protocol::BlockId,
    ctx: &LintContext,
) -> Vec<RuleViolation> {
    log::debug!("Checking signature for missing type annotations: {sig:?}");
    let params_needing_types: Vec<_> = sig
        .required_positional
        .iter()
        .chain(&sig.optional_positional)
        .chain(sig.rest_positional.iter())
        .filter(|param| param.shape == nu_protocol::SyntaxShape::Any)
        .collect();

    if params_needing_types.is_empty() {
        log::debug!("No parameters need type annotations");
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
            let param_span = signature_span.find_substring_span(&param.name, ctx);
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
