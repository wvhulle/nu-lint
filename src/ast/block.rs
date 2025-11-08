use std::collections::{HashMap, HashSet};

use nu_protocol::{
    BlockId, Span, VarId,
    ast::{Call, Expr, FindMapResult, PipelineElement, Traverse},
};

use super::{call::CallExt, pipeline::PipelineExt};
use crate::{ast::expression::ExpressionExt, context::LintContext};

pub trait BlockExt {
    /// Checks if block has side effects. Example: `{ print "hello"; ls }` has
    /// side effects
    fn has_side_effects(&self, context: &LintContext) -> bool;
    /// Checks if block is an empty list. Example: `{ [] }`
    fn is_empty_list_block(&self, context: &LintContext) -> bool;
    #[must_use]
    /// Checks if block contains a specific span. Example: function body
    /// contains statement span
    fn contains_span(&self, span: Span, context: &LintContext) -> bool;
    /// All pipeline elements: `{ ls | where size > 1kb }`
    fn all_elements<'a>(&self, context: &'a LintContext) -> Vec<&'a PipelineElement>;
    /// Tests if any pipeline element matches predicate. Example: finds `print`
    /// call
    fn any_element<F>(&self, context: &LintContext, predicate: F) -> bool
    where
        F: Fn(&PipelineElement) -> bool;
    /// Checks if block contains variable references. Example: `{ $x + 1 }`
    fn contains_variables(&self, context: &LintContext) -> bool;
    /// Extracts single `if` call from block. Example: `{ if $x { ... } }`
    fn get_single_if_call<'a>(&self, context: &'a LintContext<'a>) -> Option<&'a Call>;
    /// Checks if block contains specific command in single pipeline. Example:
    /// `{ complete }`
    fn contains_call_in_single_pipeline(&self, command_name: &str, context: &LintContext) -> bool;
    /// Checks if block contains external call with variable. Example: `{ ^$cmd
    /// args }`
    fn contains_external_call_with_variable(&self, var_id: VarId, context: &LintContext) -> bool;
    /// Collects all user function calls in block. Example: `{ foo; bar | baz }`
    /// returns `["foo", "baz"]`
    fn collect_user_function_calls(&self, context: &LintContext) -> Vec<String>;
    /// Finds all transitively called functions. Example: main calls foo, foo
    /// calls bar
    fn find_transitively_called_functions(
        &self,
        context: &LintContext,
        available_functions: &HashMap<String, BlockId>,
    ) -> HashSet<String>;
    /// Checks if block uses pipeline input variable. Example: `{ $in | length
    /// }`
    fn uses_pipeline_input(&self, context: &LintContext) -> bool;
    /// Checks if block produces output. Example: `{ ls }` produces output, `{
    /// print "x" }` doesn't
    fn produces_output(&self, context: &LintContext) -> bool;
    /// Finds the `$in` variable used in this block. Example: `def foo [] { $in
    /// | each { ... } }`
    fn find_pipeline_input_variable(&self, context: &LintContext) -> Option<VarId>;
    /// Infers the output type of a block. Example: `{ ls }` returns "table"
    fn infer_output_type(&self, context: &LintContext) -> String;
    /// Infers the input type expected by a block. Example: `{ $in | length }`
    /// expects "list"
    fn infer_input_type(&self, context: &LintContext) -> String;
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

        block
            .find_map(context.working_set, &|expr| {
                if expr.is_external_call_with_variable(var_id) {
                    FindMapResult::Found(())
                } else {
                    FindMapResult::Continue
                }
            })
            .is_some()
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

    fn uses_pipeline_input(&self, context: &LintContext) -> bool {
        let block = context.working_set.get_block(*self);
        block.pipelines.iter().any(|pipeline| {
            pipeline
                .elements
                .iter()
                .any(|element| element.expr.uses_pipeline_input(context))
        })
    }

    fn produces_output(&self, context: &LintContext) -> bool {
        let block = context.working_set.get_block(*self);
        block.pipelines.last().is_some_and(|pipeline| {
            pipeline
                .elements
                .last()
                .is_some_and(|last_element| !matches!(&last_element.expr.expr, Expr::Nothing))
        })
    }

    fn find_pipeline_input_variable(&self, context: &LintContext) -> Option<VarId> {
        let block = context.working_set.get_block(*self);
        block
            .pipelines
            .iter()
            .flat_map(|pipeline| &pipeline.elements)
            .find_map(|element| element.expr.find_pipeline_input_variable(context))
    }

    fn infer_output_type(&self, context: &LintContext) -> String {
        let block = context.working_set.get_block(*self);
        log::debug!("Inferring output type for block {self:?}");

        block
            .pipelines
            .last()
            .and_then(|pipeline| pipeline.elements.last())
            .and_then(|elem| elem.expr.infer_output_type(context))
            .unwrap_or_else(|| block.output_type().to_string())
    }

    fn infer_input_type(&self, context: &LintContext) -> String {
        use super::expression::ExpressionExt;

        let block = context.working_set.get_block(*self);
        let Some(in_var) = self.find_pipeline_input_variable(context) else {
            return "any".to_string();
        };

        block
            .pipelines
            .iter()
            .flat_map(|pipeline| &pipeline.elements)
            .find_map(|element| element.expr.infer_input_type(Some(in_var), context))
            .map_or_else(|| "any".to_string(), String::from)
    }
}
