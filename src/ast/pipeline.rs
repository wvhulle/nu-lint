use nu_protocol::{
    Type, VarId,
    ast::{Expr, Pipeline, PipelineElement},
};

use crate::{ast::expression::ExpressionExt, context::LintContext};

pub trait PipelineExt {
    /// Infers parameter type from pipeline. Example: `$text | str length`
    /// infers `string`
    fn infer_param_type(&self, param_var_id: VarId, context: &LintContext) -> Option<Type>;
}

impl PipelineExt for Pipeline {
    fn infer_param_type(&self, param_var_id: VarId, context: &LintContext) -> Option<Type> {
        log::debug!(
            "infer_param_type from pipeline: param_var_id={:?}, pipeline_elements={}",
            param_var_id,
            self.elements.len()
        );

        let result = self
            .elements
            .windows(2)
            .find_map(|window| infer_from_pipeline_window(param_var_id, window, context));

        log::debug!("infer_param_type from pipeline result: {result:?}");
        result
    }
}

fn infer_from_pipeline_window(
    param_var_id: VarId,
    window: &[PipelineElement],
    context: &LintContext,
) -> Option<Type> {
    let contains_param = window[0].expr.contains_variable(param_var_id);
    log::debug!(
        "  Checking pipeline window: contains_param={}, first_expr={:?}, second_expr={:?}",
        contains_param,
        &window[0].expr.expr,
        &window[1].expr.expr
    );

    let Expr::Call(call) = &window[1].expr.expr else {
        log::debug!("  -> Not a call expression");
        return None;
    };

    if !contains_param {
        log::debug!("  -> Parameter not used in first element");
        return None;
    }

    let decl = context.working_set.get_decl(call.decl_id);
    let sig = decl.signature();

    let Some((input_type, _)) = sig.input_output_types.first() else {
        log::debug!("  -> No input/output types for '{}'", decl.name());
        return None;
    };

    if matches!(input_type, Type::Any) {
        log::debug!(
            "  -> Found call to '{}', but input_type is Any",
            decl.name()
        );
        return None;
    }

    log::debug!(
        "  -> Found call to '{}', input_type={:?}",
        decl.name(),
        input_type
    );
    Some(input_type.clone())
}
