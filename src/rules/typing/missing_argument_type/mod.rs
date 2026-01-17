use nu_protocol::{
    BlockId, Span, Type, VarId,
    ast::{Call, Expr},
};

use crate::{
    LintLevel,
    ast::{
        block::BlockExt, call::CallExt, expression::ExpressionExt, pipeline::PipelineExt,
        span::SpanExt,
    },
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

/// Semantic fix data: stores signature span and body block ID for regenerating
/// the fix
pub struct FixData {
    signature_span: Span,
    body_block_id: BlockId,
}

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

    // Fall back to expression-based inference (handles arguments, closures, binary
    // ops, etc.)
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
        shape.to_string() // Use upstream Display
    }
}

fn detect_signature(
    sig: &nu_protocol::Signature,
    signature_span: Span,
    body_block_id: BlockId,
    ctx: &LintContext,
) -> Vec<(Detection, FixData)> {
    log::debug!("Checking signature for missing type annotations: {sig:?}");
    let block = ctx.working_set.get_block(body_block_id);

    let params_needing_types: Vec<_> = sig
        .required_positional
        .iter()
        .chain(&sig.optional_positional)
        .chain(sig.rest_positional.iter())
        .filter(|param| param.shape == nu_protocol::SyntaxShape::Any)
        .map(|param| {
            (
                param,
                param
                    .var_id
                    .map(|var_id| block.var_usages(var_id, ctx, |_, _, _| true)),
            )
        })
        .collect();

    if params_needing_types.is_empty() {
        log::debug!("No parameters need type annotations");
        return vec![];
    }

    params_needing_types
        .into_iter()
        .map(|(param, usage_span)| {
            let param_span = signature_span.find_substring_span(&param.name, ctx);
            let mut violation = Detection::from_global_span(
                format!("Parameter `{}` is missing type annotation", param.name),
                param_span,
            )
            .with_primary_label("add type annotation");

            if let Some(usage_spans) = usage_span
                && let Some(&first_span) = usage_spans.first() {
                    violation = violation.with_extra_label("used here", first_span);
                }

            let fix_data = FixData {
                signature_span,
                body_block_id,
            };

            (violation, fix_data)
        })
        .collect()
}

fn detect_def_call(call: &Call, ctx: &LintContext) -> Vec<(Detection, FixData)> {
    call.custom_command_def(ctx)
        .is_some()
        .then(|| {
            call.get_positional_arg(1)
                .zip(call.get_positional_arg(2))
                .and_then(|(sig_arg, body_arg)| {
                    body_arg
                        .extract_block_id()
                        .and_then(|body_block_id| match &sig_arg.expr {
                            Expr::Signature(sig) => {
                                Some(detect_signature(sig, sig_arg.span, body_block_id, ctx))
                            }
                            _ => None,
                        })
                })
        })
        .flatten()
        .unwrap_or_default()
}

struct MissingTypeAnnotation;

impl DetectFix for MissingTypeAnnotation {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "add_type_hints_arguments"
    }

    fn short_description(&self) -> &'static str {
        "Arguments of custom commands should have type annotations"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/custom_commands.html#parameter-types")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| match &expr.expr {
            Expr::Call(call) => detect_def_call(call, ctx),
            _ => vec![],
        })
    }

    fn fix(&self, ctx: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let block = ctx.working_set.get_block(fix_data.body_block_id);
        let body_block_id = fix_data.body_block_id;
        let ctx: &LintContext = ctx;
        let params = block
            .signature
            .required_positional
            .iter()
            .map(|p| {
                format!(
                    "{}: {}",
                    p.name,
                    get_param_type_str(&p.shape, p.var_id, body_block_id, ctx)
                )
            })
            .chain(block.signature.optional_positional.iter().map(|p| {
                format!(
                    "{}?: {}",
                    p.name,
                    get_param_type_str(&p.shape, p.var_id, body_block_id, ctx)
                )
            }))
            .chain(block.signature.rest_positional.iter().map(|p| {
                format!(
                    "...{}: {}",
                    p.name,
                    get_param_type_str(&p.shape, p.var_id, body_block_id, ctx)
                )
            }))
            .collect::<Vec<_>>()
            .join(", ");

        let new_sig = { format!("[{params}]") };
        Some(Fix::with_explanation(
            "Add type annotations to parameters",
            vec![Replacement::new(fix_data.signature_span, new_sig)],
        ))
    }
}

pub static RULE: &dyn Rule = &MissingTypeAnnotation;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
