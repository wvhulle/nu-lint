use nu_protocol::{
    Type, VarId,
    ast::{Expr, Expression, Pipeline},
};

use crate::{
    ast::{call::CallExt, expression::ExpressionExt, span::SpanExt},
    context::LintContext,
};

pub trait PipelineExt {
    /// Checks if pipeline contains call to command. Example: `ls | where size >
    /// 1kb` contains "where"
    fn contains_call_to(&self, command_name: &str, context: &LintContext) -> bool;
    /// Checks if pipeline contains indexed access. Example: `split row ":" |
    /// get 0`
    fn contains_indexed_access(&self, context: &LintContext) -> bool;
    /// Checks if variable is used in pipeline. Example: `$list | length` uses
    /// `$list`
    fn variable_is_used(&self, var_id: VarId) -> bool;
    /// Checks if variable is piped. Example: `$data | to json` pipes `$data`
    fn variable_is_piped(&self, var_id: VarId) -> bool;
    /// Checks if pipeline ends with ignore. Example: `ls | ignore`
    fn ends_with_ignore(&self, context: &LintContext) -> bool;
    /// Gets element before ignore. Example: `mkdir tmp | ignore` returns `mkdir
    /// tmp`
    fn element_before_ignore(&self, context: &LintContext) -> Option<&Expression>;
    /// Infers parameter type from pipeline. Example: `$text | str length`
    /// infers `string`
    fn infer_param_type(&self, param_var_id: VarId, context: &LintContext) -> Option<Type>;
}

impl PipelineExt for Pipeline {
    fn contains_call_to(&self, command_name: &str, context: &LintContext) -> bool {
        fn check_expr_for_command(expr: &Expr, command_name: &str, context: &LintContext) -> bool {
            match expr {
                Expr::Call(call) if call.is_call_to_command(command_name, context) => true,
                Expr::FullCellPath(cp) => {
                    check_expr_for_command(&cp.head.expr, command_name, context)
                }
                Expr::Subexpression(block_id) => {
                    let block = context.working_set.get_block(*block_id);
                    block
                        .pipelines
                        .iter()
                        .any(|p| p.contains_call_to(command_name, context))
                }
                _ => false,
            }
        }

        self.elements
            .iter()
            .any(|element| check_expr_for_command(&element.expr.expr, command_name, context))
    }

    fn contains_indexed_access(&self, context: &LintContext) -> bool {
        self.elements.iter().any(|element| {
            let Expr::Call(call) = &element.expr.expr else {
                return false;
            };

            let name = call.get_call_name(context);
            matches!(name.as_str(), "get" | "skip")
                && call.get_first_positional_arg().is_some_and(|arg| {
                    let arg_text = arg.span.text(context);
                    arg_text.parse::<usize>().is_ok()
                })
        })
    }

    fn variable_is_used(&self, var_id: VarId) -> bool {
        self.elements.iter().any(|elem| match &elem.expr.expr {
            Expr::Var(v_id) if *v_id == var_id => true,
            Expr::FullCellPath(cp) => matches!(&cp.head.expr, Expr::Var(v_id) if *v_id == var_id),
            _ => false,
        })
    }

    fn variable_is_piped(&self, var_id: VarId) -> bool {
        if self.elements.is_empty() {
            return false;
        }

        let first = &self.elements[0];
        matches!(&first.expr.expr, Expr::FullCellPath(cell_path)
            if matches!(&cell_path.head.expr, Expr::Var(ref_var_id) if *ref_var_id == var_id)
            && cell_path.tail.is_empty())
    }

    fn ends_with_ignore(&self, context: &LintContext) -> bool {
        self.elements.last().is_some_and(|elem| {
            matches!(&elem.expr.expr, Expr::Call(call) if call.is_call_to_command("ignore", context))
        })
    }

    fn element_before_ignore(&self, context: &LintContext) -> Option<&Expression> {
        (self.elements.len() >= 2 && self.ends_with_ignore(context))
            .then(|| &self.elements[self.elements.len() - 2].expr)
    }

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

#[allow(
    clippy::absolute_paths,
    reason = "PipelineElement is not exposed in public API"
)]
fn infer_from_pipeline_window(
    param_var_id: VarId,
    window: &[nu_protocol::ast::PipelineElement],
    context: &LintContext,
) -> Option<Type> {
    // Check if first element uses the parameter variable
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

    // Get the input type from the signature's input_output_types
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
