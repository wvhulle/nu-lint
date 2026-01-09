use nu_protocol::{
    Type,
    ast::{Call, Expr},
};

use super::{
    FixData, extract_parameters_text, find_return_span, find_signature_span, get_input_type,
    get_output_type,
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
        "Checking function definition for type_command_output: {}",
        def.name
    );

    let block = ctx.working_set.get_block(def.body);
    let signature = &block.signature;

    if !block.produces_output() {
        return vec![];
    }

    let inferred_output = block.infer_output_type(ctx);
    if matches!(inferred_output, Type::Nothing) {
        return vec![];
    }

    let sig_span = find_signature_span(call, ctx);
    let output_type = get_output_type(signature);

    let needs_refinement = match output_type {
        None | Some(Type::Any) => true,
        Some(_) => false,
    };

    if !needs_refinement {
        return vec![];
    }

    let Some(sig_span) = sig_span else {
        return vec![];
    };

    let return_span = find_return_span(block);
    let fix_data = FixData {
        sig_span,
        body_block_id: def.body,
    };

    let mut violation = Detection::from_global_span(
        format!("'{}' missing or using 'any' for output type", def.name),
        def.name_span,
    )
    .with_primary_label("add specific output type");

    if let Some(span) = return_span {
        violation = violation.with_extra_label("returned here", span);
    }

    vec![(violation, fix_data)]
}

struct TypeCommandOutput;

impl DetectFix for TypeCommandOutput {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "missing_output_type"
    }

    fn short_description(&self) -> &'static str {
        "Custom commands that produce output should have specific output type annotations"
    }

    fn source_link(&self) -> Option<&'static str> {
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
        let signature = &block.signature;

        let params = extract_parameters_text(signature);
        let input_type = get_input_type(signature).unwrap_or(Type::Any);
        let output_type = block.infer_output_type(ctx);

        let new_signature = format!("[{params}]: {input_type} -> {output_type}");

        Some(Fix::with_explanation(
            format!("Add output type annotation: {new_signature}"),
            vec![Replacement::new(fix_data.sig_span, new_signature)],
        ))
    }
}

pub static RULE: &dyn Rule = &TypeCommandOutput;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
