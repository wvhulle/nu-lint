use nu_protocol::{
    BlockId, PositionalArg, Span, SyntaxShape, Type, VarId,
    ast::{Call, Expr},
};

use crate::{
    LintLevel,
    ast::{
        block::BlockExt, call::CallExt, declaration::CustomCommandDef, expression::ExpressionExt,
        pipeline::PipelineExt,
    },
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

pub struct FixData {
    edits: Vec<AnnotationEdit>,
    body_block_id: BlockId,
}

fn infer_param_type(
    param_var_id: VarId,
    body_block_id: nu_protocol::BlockId,
    ctx: &LintContext,
) -> Type {
    log::trace!("infer_param_type: param_var_id={param_var_id:?}, body_block_id={body_block_id:?}");
    let block = ctx.working_set.get_block(body_block_id);

    // First try pipeline-based inference
    log::trace!("  Trying pipeline-based inference...");
    let pipeline_type = block
        .pipelines
        .iter()
        .find_map(|pipeline| pipeline.infer_param_type(param_var_id, ctx));

    if let Some(ty) = &pipeline_type {
        log::trace!("  -> Pipeline-based inference found: {ty:?}");
        return ty.clone();
    }

    // Fall back to expression-based inference (handles arguments, closures, binary
    // ops, etc.)
    log::trace!("  Trying expression-based inference...");
    let expr_type = block
        .pipelines
        .iter()
        .flat_map(|pipeline| &pipeline.elements)
        .find_map(|element| {
            let result = element.expr.infer_input_type(Some(param_var_id), ctx);
            log::trace!("    Checked element, result: {result:?}");
            result
        });

    if let Some(ty) = &expr_type {
        log::trace!("  -> Expression-based inference found: {ty:?}");
        return ty.clone();
    }

    log::trace!("  -> No type found, returning Type::Any");
    Type::Any
}

#[derive(Clone, Copy)]
enum AnnotationEdit {
    InferType { name_span: Span, var_id: VarId },
    InferRestElementType { name_span: Span, var_id: VarId },
}

struct MissingAnnotation<'a> {
    param: &'a PositionalArg,
    name_span: Span,
    edit: AnnotationEdit,
}

fn declaration_span(var_id: VarId, ctx: &LintContext) -> Span {
    ctx.working_set.get_variable(var_id).declaration_span
}

fn untyped_params<'a>(
    def: &'a CustomCommandDef,
    ctx: &LintContext,
) -> impl Iterator<Item = MissingAnnotation<'a>> {
    let sig = &def.signature;
    let positionals = sig
        .required_positional
        .iter()
        .chain(&sig.optional_positional)
        .map(|param| (param, false));
    let rest = sig
        .rest_positional
        .iter()
        .filter(|_| !def.is_wrapped)
        .map(|param| (param, true));
    positionals
        .chain(rest)
        .filter(|(param, _)| param.shape == SyntaxShape::Any)
        .filter_map(move |(param, is_rest)| {
            let var_id = param.var_id?;
            let name_span = declaration_span(var_id, ctx);
            let edit = if is_rest {
                AnnotationEdit::InferRestElementType { name_span, var_id }
            } else {
                AnnotationEdit::InferType { name_span, var_id }
            };
            Some(MissingAnnotation {
                param,
                name_span,
                edit,
            })
        })
}

fn rest_element_type(list_type: Type) -> Type {
    match list_type {
        Type::List(element) => *element,
        _ => Type::Any,
    }
}

fn insert_type_after(name_span: Span, ty: &Type) -> Replacement {
    Replacement::new(Span::new(name_span.end, name_span.end), format!(": {ty}"))
}

fn to_detection(
    missing: &MissingAnnotation,
    body_block_id: BlockId,
    ctx: &LintContext,
) -> Detection {
    let detection = Detection::from_global_span(
        format!(
            "Parameter `{}` is missing type annotation",
            missing.param.name
        ),
        missing.name_span,
    )
    .with_primary_label("add type annotation");
    let first_usage = missing.param.var_id.and_then(|var_id| {
        ctx.working_set
            .get_block(body_block_id)
            .var_usages(var_id, ctx)
            .first()
            .copied()
    });
    match first_usage {
        Some(usage_span) => detection.with_extra_label("used here", usage_span),
        None => detection,
    }
}

fn detect_def_call(call: &Call, ctx: &LintContext) -> Vec<(Detection, FixData)> {
    let Some(def) = call.custom_command_def(ctx) else {
        return vec![];
    };
    let missing: Vec<_> = untyped_params(&def, ctx).collect();
    let edits: Vec<_> = missing.iter().map(|missing| missing.edit).collect();
    missing
        .iter()
        .map(|missing| {
            (
                to_detection(missing, def.body, ctx),
                FixData {
                    edits: edits.clone(),
                    body_block_id: def.body,
                },
            )
        })
        .collect()
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
        let replacements = fix_data
            .edits
            .iter()
            .map(|edit| match *edit {
                AnnotationEdit::InferType { name_span, var_id } => insert_type_after(
                    name_span,
                    &infer_param_type(var_id, fix_data.body_block_id, ctx),
                ),
                AnnotationEdit::InferRestElementType { name_span, var_id } => insert_type_after(
                    name_span,
                    &rest_element_type(infer_param_type(var_id, fix_data.body_block_id, ctx)),
                ),
            })
            .collect();
        Some(Fix {
            explanation: "Add type annotations to parameters".into(),
            replacements,
        })
    }
}

pub static RULE: &dyn Rule = &MissingTypeAnnotation;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
