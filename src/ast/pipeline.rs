use nu_protocol::{
    VarId,
    ast::{Expr, Pipeline},
};

use super::{CallExt, SpanExt};
use crate::context::LintContext;

pub trait PipelineExt {
    fn contains_call_to(&self, command_name: &str, context: &LintContext) -> bool;
    fn contains_indexed_access(&self, context: &LintContext) -> bool;
    fn variable_is_used(&self, var_id: VarId) -> bool;
    fn variable_is_piped(&self, var_id: VarId) -> bool;
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
}
