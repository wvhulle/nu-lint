use nu_lint_analysis::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::{
    Type,
    ast::{Call, Expr},
};

use super::{FixData, find_signature_span, get_input_type, get_output_type, parameter_list_text};

fn detect_def_call(call: &Call, ctx: &LintContext) -> Vec<(Detection, FixData)> {
    let Some(def) = call.custom_command_def(ctx) else {
        return vec![];
    };

    log::trace!(
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
        signature_is_annotated: !def.signature.input_output_types.is_empty(),
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

    fn short_description(&self) -> &'static str {
        "Command using `$in` missing input type"
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

        // Use AST data: extract parameters and types from signature
        let parameter_list = parameter_list_text(fix_data.sig_span, ctx)?;
        let input_type = block.infer_input_type(ctx);
        if input_type == Type::Any && fix_data.signature_is_annotated {
            return None;
        }
        let output_type = get_output_type(signature).unwrap_or(Type::Any);

        let new_signature = format!("{parameter_list}: {input_type} -> {output_type}");

        Some(Fix {
            explanation: format!("Add input type annotation: {new_signature}").into(),
            replacements: vec![Replacement::new(fix_data.sig_span, new_signature)],
        })
    }
}

pub static RULE: &dyn Rule = &TypeNuPipelineInput;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
