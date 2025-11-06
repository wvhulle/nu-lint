use std::collections::{HashMap, HashSet};

use nu_protocol::{
    BlockId, Span, VarId,
    ast::{Call, Expr, PipelineElement, Traverse},
};

use super::{call::CallExt, pipeline::PipelineExt};
use crate::{ast::expression::ExpressionExt, context::LintContext};

pub trait BlockExt {
    fn has_side_effects(&self, context: &LintContext) -> bool;
    fn is_empty_list_block(&self, context: &LintContext) -> bool;
    #[must_use]
    fn contains_span(&self, span: Span, context: &LintContext) -> bool;
    fn all_elements<'a>(&self, context: &'a LintContext) -> Vec<&'a PipelineElement>;
    fn any_element<F>(&self, context: &LintContext, predicate: F) -> bool
    where
        F: Fn(&PipelineElement) -> bool;
    fn contains_variables(&self, context: &LintContext) -> bool;
    fn get_single_if_call<'a>(&self, context: &'a LintContext<'a>) -> Option<&'a Call>;
    fn contains_call_in_single_pipeline(&self, command_name: &str, context: &LintContext) -> bool;
    fn contains_external_call_with_variable(&self, var_id: VarId, context: &LintContext) -> bool;
    fn collect_user_function_calls(&self, context: &LintContext) -> Vec<String>;
    fn find_transitively_called_functions(
        &self,
        context: &LintContext,
        available_functions: &HashMap<String, BlockId>,
    ) -> HashSet<String>;
}

impl BlockExt for BlockId {
    fn has_side_effects(&self, context: &LintContext) -> bool {
        self.all_elements(context)
            .iter()
            .any(|elem| !elem.expr.is_likely_pure())
    }

    fn is_empty_list_block(&self, context: &LintContext) -> bool {
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

    fn any_element<F>(&self, context: &LintContext, predicate: F) -> bool
    where
        F: Fn(&PipelineElement) -> bool,
    {
        self.all_elements(context).iter().any(|e| predicate(e))
    }

    fn contains_variables(&self, context: &LintContext) -> bool {
        self.any_element(context, |elem| elem.expr.contains_variables(context))
    }

    fn contains_call_in_single_pipeline(&self, command_name: &str, context: &LintContext) -> bool {
        let block = context.working_set.get_block(*self);
        block.pipelines.len() == 1
            && block
                .pipelines
                .first()
                .is_some_and(|p| p.contains_call_to(command_name, context))
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

    fn collect_user_function_calls(&self, context: &LintContext) -> Vec<String> {
        let block = context.working_set.get_block(*self);
        let mut function_calls = Vec::new();

        block.flat_map(
            context.working_set,
            &|expr| {
                if let Expr::Call(call) = &expr.expr {
                    vec![call.get_call_name(context)]
                } else {
                    vec![]
                }
            },
            &mut function_calls,
        );

        function_calls
    }

    fn find_transitively_called_functions(
        &self,
        context: &LintContext,
        available_functions: &HashMap<String, BlockId>,
    ) -> HashSet<String> {
        self.collect_user_function_calls(context)
            .into_iter()
            .filter_map(|func_name| {
                available_functions.get(&func_name).map(|&callee_block_id| {
                    let mut transitive = callee_block_id
                        .find_transitively_called_functions(context, available_functions);
                    transitive.insert(func_name);
                    transitive
                })
            })
            .flatten()
            .collect()
    }
}
