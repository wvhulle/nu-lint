use nu_protocol::{
    BlockId, Span, VarId,
    ast::{Call, Expr, PipelineElement},
};

use super::{CallExt, PipelineExt};
use crate::context::LintContext;

pub trait BlockExt {
    fn has_side_effects(&self, context: &LintContext) -> bool;
    fn is_empty_list_block(&self, context: &LintContext) -> bool;
    #[must_use]
    fn contains_span(&self, span: Span, context: &LintContext) -> bool;
    fn all_elements<'a>(&self, context: &'a LintContext) -> Vec<&'a PipelineElement>;
    fn contains_call_to(&self, command_name: &str, context: &LintContext) -> bool;
    fn any_element<F>(&self, context: &LintContext, predicate: F) -> bool
    where
        F: Fn(&PipelineElement) -> bool;
    fn contains_variables(&self, context: &LintContext) -> bool;
    fn get_single_if_call<'a>(&self, context: &'a LintContext<'a>) -> Option<&'a Call>;
    fn contains_call_in_single_pipeline(&self, command_name: &str, context: &LintContext) -> bool;
    fn contains_external_call_with_variable(&self, var_id: VarId, context: &LintContext) -> bool;
}

impl BlockExt for BlockId {
    fn has_side_effects(&self, context: &LintContext) -> bool {
        use super::ExpressionExt;

        self.all_elements(context)
            .iter()
            .any(|elem| !elem.expr.is_likely_pure())
    }

    fn is_empty_list_block(&self, context: &LintContext) -> bool {
        use super::ExpressionExt;

        let block = context.working_set.get_block(*self);

        block
            .pipelines
            .first()
            .and_then(|pipeline| pipeline.elements.first())
            .is_some_and(|elem| elem.expr.is_empty_list())
    }

    fn contains_span(&self, span: Span, context: &LintContext) -> bool {
        let block = context.working_set.get_block(*self);
        if let Some(block_span) = block.span {
            return span.start >= block_span.start && span.end <= block_span.end;
        }
        false
    }

    fn all_elements<'a>(&self, context: &'a LintContext) -> Vec<&'a PipelineElement> {
        let block = context.working_set.get_block(*self);
        block.pipelines.iter().flat_map(|p| &p.elements).collect()
    }

    fn contains_call_to(&self, command_name: &str, context: &LintContext) -> bool {
        use super::ExpressionExt;

        self.all_elements(context)
            .iter()
            .any(|elem| elem.expr.is_call_to(command_name, context))
    }

    fn any_element<F>(&self, context: &LintContext, predicate: F) -> bool
    where
        F: Fn(&PipelineElement) -> bool,
    {
        self.all_elements(context).iter().any(|e| predicate(e))
    }

    fn contains_variables(&self, context: &LintContext) -> bool {
        use super::ExpressionExt;

        self.all_elements(context)
            .iter()
            .any(|elem| elem.expr.contains_variables(context))
    }

    fn contains_call_in_single_pipeline(&self, command_name: &str, context: &LintContext) -> bool {
        let block = context.working_set.get_block(*self);
        block.pipelines.len() == 1 && block.pipelines[0].contains_call_to(command_name, context)
    }

    fn get_single_if_call<'a>(&self, context: &'a LintContext<'a>) -> Option<&'a Call> {
        let block = context.working_set.get_block(*self);

        let pipeline = (block.pipelines.len() == 1).then(|| block.pipelines.first())??;

        let element = (pipeline.elements.len() == 1).then(|| pipeline.elements.first())??;

        match &element.expr.expr {
            Expr::Call(call) if call.is_call_to_command("if", context) => Some(call),
            _ => None,
        }
    }

    fn contains_external_call_with_variable(&self, var_id: VarId, context: &LintContext) -> bool {
        use nu_protocol::ast::Traverse;

        use super::ExpressionExt;

        let block = context.working_set.get_block(*self);
        let mut results = Vec::new();

        block.flat_map(
            context.working_set,
            &|expr| {
                if expr.is_external_call_with_variable(var_id) {
                    vec![true]
                } else {
                    vec![]
                }
            },
            &mut results,
        );

        !results.is_empty()
    }
}
