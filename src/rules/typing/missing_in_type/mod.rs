use nu_protocol::{
    Type,
    ast::{Call, Expr},
};

use super::{
    FixData, extract_parameters_text, find_signature_span, get_input_type, parse_signature_types,
};
use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

fn detect_def_call(call: &Call, ctx: &LintContext) -> Vec<(Detection, FixData)> {
    let Some(def) = call.custom_command_def(ctx) else {
        return vec![];
    };

    log::debug!(
        "Checking function definition for type_nu_pipeline_input: {}",
        def.name
    );

    let block = ctx.working_set.get_block(def.body);
    let signature = &block.signature;

    if !block.uses_pipeline_input(ctx) {
        return vec![];
    }

    let sig_span = find_signature_span(call, ctx);
    let input_type = get_input_type(signature);

    let needs_refinement = match input_type {
        None | Some(Type::Any) => true,
        Some(_) => false,
    };

    if !needs_refinement {
        return vec![];
    }

    let Some(sig_span) = sig_span else {
        return vec![];
    };

    let in_usage_span = block.find_dollar_in_usage();
    let fix_data = FixData {
        sig_span,
        body_block_id: def.body,
    };

    let mut violation = Detection::from_global_span(
        format!("'{}' missing or using 'any' for input type", def.name),
        def.name_span,
    )
    .with_primary_label("add specific input type");

    if let Some(span) = in_usage_span {
        violation = violation.with_extra_label("$in used here", span);
    }

    vec![(violation, fix_data)]
}

struct TypeNuPipelineInput;

impl DetectFix for TypeNuPipelineInput {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "missing_in_type"
    }

    fn explanation(&self) -> &'static str {
        "Custom commands that use $in should have specific input type annotations"
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
        let original_sig_text = ctx.plain_text(fix_data.sig_span);
        let parsed = parse_signature_types(original_sig_text);
        let params = extract_parameters_text(&block.signature);

        let input_type = block.infer_input_type(ctx);
        let output_type = parsed.output_type.unwrap_or(Type::Any);

        let new_signature = format!("[{params}]: {input_type} -> {output_type}");

        Some(Fix::with_explanation(
            format!("Add input type annotation: {new_signature}"),
            vec![Replacement::new(fix_data.sig_span, new_signature)],
        ))
    }
}

pub static RULE: &dyn Rule = &TypeNuPipelineInput;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
