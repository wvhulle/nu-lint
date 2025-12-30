use std::collections::{HashMap, HashSet};

use nu_protocol::{
    BlockId, Span, Type, VarId,
    ast::{Block, Expr, Expression, PipelineElement, Traverse},
};

use super::call::CallExt;
use crate::{ast::expression::ExpressionExt, context::LintContext};

const MAX_TYPE_INFERENCE_DEPTH: usize = 100;

fn find_transitively_called_functions_impl(
    block: &Block,
    context: &LintContext,
    available_functions: &HashMap<String, BlockId>,
    visited: &mut HashSet<usize>,
) -> HashSet<String> {
    // Prevent infinite recursion on recursive/mutually recursive functions
    // We track visited blocks by comparing their memory addresses
    #[allow(
        clippy::ref_as_ptr,
        reason = "Need pointer address as unique identifier for cycle detection"
    )]
    let block_ptr = block as *const Block as usize;

    if !visited.insert(block_ptr) {
        log::debug!("Cycle detected in function calls");
        return HashSet::new();
    }

    block
        .collect_user_function_calls(context)
        .into_iter()
        .filter_map(|func_name| {
            available_functions.get(&func_name).map(|&callee_block_id| {
                let callee_block = context.working_set.get_block(callee_block_id);
                let mut transitive = find_transitively_called_functions_impl(
                    callee_block,
                    context,
                    available_functions,
                    visited,
                );
                transitive.insert(func_name);
                transitive
            })
        })
        .flatten()
        .collect()
}

fn infer_output_type_with_depth(block: &Block, context: &LintContext, depth: usize) -> Type {
    if depth >= MAX_TYPE_INFERENCE_DEPTH {
        log::warn!(
            "Type inference depth limit ({MAX_TYPE_INFERENCE_DEPTH}) reached, returning Any"
        );
        return Type::Any;
    }

    log::debug!("Inferring output type for block (depth={depth})");

    let Some(pipeline) = block.pipelines.last() else {
        return block.output_type();
    };

    let block_input_type = block
        .all_elements()
        .iter()
        .find_map(|element| element.expr.find_pipeline_input(context))
        .and_then(|(in_var, _)| {
            block
                .all_elements()
                .iter()
                .find_map(|element| element.expr.infer_input_type(Some(in_var), context))
        })
        .unwrap_or(Type::Any);
    log::debug!("Block inferred input type: {block_input_type:?}");
    let mut current_type = Some(block_input_type);

    for (idx, element) in pipeline.elements.iter().enumerate() {
        log::debug!("Pipeline element {idx}: current_type before = {current_type:?}");

        if let Expr::Call(call) = &element.expr.expr {
            let output = call.get_output_type(context, current_type);
            log::debug!("Pipeline element {idx} (Call): output type = {output:?}");
            current_type = Some(output);
            continue;
        }

        let inferred = element.expr.infer_output_type(context);
        log::debug!("Pipeline element {idx} (Expression): inferred type = {inferred:?}");
        if inferred.is_some() {
            current_type = inferred;
        }
    }

    let final_type = current_type.unwrap_or_else(|| block.output_type());
    log::debug!("Block final output type: {final_type:?}");
    final_type
}

pub trait BlockExt {
    /// Checks if block is an empty list. Example: `{ [] }`
    fn is_empty_list_block(&self) -> bool;
    #[must_use]
    /// Checks if block contains a specific span. Example: function body
    /// contains statement span
    fn contains_span(&self, span: Span) -> bool;
    /// All pipeline elements: `{ ls | where size > 1kb }`
    fn all_elements(&self) -> Vec<&PipelineElement>;
    /// Checks if block contains variable references. Example: `{ $x + 1 }`
    fn contains_variables(&self, context: &LintContext) -> bool;
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
    fn produces_output(&self) -> bool;
    /// Finds pipeline input-like variables (includes `$in` and closure
    /// parameters) and their spans. Example: `{ $in | length }` returns
    /// `(var_id, span of $in)`
    fn find_pipeline_input(&self, context: &LintContext) -> Option<(VarId, Span)>;
    /// Finds the actual `$in` variable usage and its span. Example: `{ $in |
    /// length }` returns span of `$in`. Does not match closure parameters.
    fn find_dollar_in_usage(&self) -> Option<Span>;
    /// Finds the first usage span of a specific variable in this block.
    /// Example: `{ $x + 1 }` with `var_id` of x returns span of `$x`
    fn find_var_usage(&self, var_id: VarId) -> Option<Span>;
    /// Infers the output type of a block. Example: `{ ls }` returns "table"
    fn infer_output_type(&self, context: &LintContext) -> Type;
    /// Infers the input type expected by a block. Example: `{ $in | length }`
    /// expects "list"
    fn infer_input_type(&self, context: &LintContext) -> Type;
    /// Extracts variable IDs that are assigned to within a block. Example: `{
    /// $x = 5; $y += 1 }` returns [x, y]
    fn extract_assigned_vars(&self) -> Vec<VarId>;

    /// Finds spans of variable usages matching a predicate. Example: finding
    /// all usages of `$x` that are inside null checks
    fn find_var_usage_spans<F>(
        &self,
        var_id: VarId,
        context: &LintContext,
        predicate: F,
    ) -> Vec<Span>
    where
        F: Fn(&Expression, VarId, &LintContext) -> bool;

    /// Finds spans of expressions matching a predicate. Example: finding all
    /// expressions that contain null checks for a variable
    fn find_expr_spans<F>(&self, context: &LintContext, predicate: F) -> Vec<Span>
    where
        F: Fn(&Expression, &LintContext) -> bool;
}

impl BlockExt for Block {
    fn is_empty_list_block(&self) -> bool {
        self.pipelines
            .first()
            .and_then(|pipeline| pipeline.elements.first())
            .is_some_and(|elem| elem.expr.is_empty_list())
    }

    fn contains_span(&self, span: Span) -> bool {
        if let Some(block_span) = self.span {
            return span.start >= block_span.start && span.end <= block_span.end;
        }
        false
    }

    fn all_elements(&self) -> Vec<&PipelineElement> {
        self.pipelines.iter().flat_map(|p| &p.elements).collect()
    }

    fn contains_variables(&self, context: &LintContext) -> bool {
        self.all_elements()
            .iter()
            .any(|elem| elem.expr.contains_variables(context))
    }

    fn collect_user_function_calls(&self, context: &LintContext) -> Vec<String> {
        let mut function_calls = Vec::new();

        self.flat_map(
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
        let mut visited: HashSet<usize> = HashSet::new();
        find_transitively_called_functions_impl(self, context, available_functions, &mut visited)
    }

    fn uses_pipeline_input(&self, context: &LintContext) -> bool {
        self.all_elements()
            .iter()
            .any(|elem| elem.expr.uses_pipeline_input(context))
    }

    fn produces_output(&self) -> bool {
        self.pipelines.last().is_some_and(|pipeline| {
            pipeline
                .elements
                .last()
                .is_some_and(|last_element| !matches!(&last_element.expr.expr, Expr::Nothing))
        })
    }

    fn find_pipeline_input(&self, context: &LintContext) -> Option<(VarId, Span)> {
        self.all_elements()
            .iter()
            .find_map(|element| element.expr.find_pipeline_input(context))
    }

    fn find_dollar_in_usage(&self) -> Option<Span> {
        self.all_elements()
            .iter()
            .find_map(|element| element.expr.find_dollar_in_usage())
    }

    fn find_var_usage(&self, var_id: VarId) -> Option<Span> {
        self.all_elements()
            .iter()
            .find_map(|element| element.expr.find_var_usage(var_id))
    }

    fn infer_output_type(&self, context: &LintContext) -> Type {
        infer_output_type_with_depth(self, context, 0)
    }

    fn infer_input_type(&self, context: &LintContext) -> Type {
        let Some((in_var, _)) = self.find_pipeline_input(context) else {
            return Type::Any;
        };

        self.all_elements()
            .iter()
            .find_map(|element| element.expr.infer_input_type(Some(in_var), context))
            .unwrap_or(Type::Any)
    }

    fn extract_assigned_vars(&self) -> Vec<VarId> {
        self.all_elements()
            .iter()
            .filter_map(|elem| elem.expr.extract_assigned_variable())
            .collect()
    }

    fn find_var_usage_spans<F>(
        &self,
        var_id: VarId,
        context: &LintContext,
        predicate: F,
    ) -> Vec<Span>
    where
        F: Fn(&Expression, VarId, &LintContext) -> bool,
    {
        use nu_protocol::ast::Expression;

        let mut matching_spans = Vec::new();
        self.flat_map(
            context.working_set,
            &|expr: &Expression| {
                if expr.matches_var(var_id) && predicate(expr, var_id, context) {
                    vec![expr.span]
                } else {
                    vec![]
                }
            },
            &mut matching_spans,
        );
        matching_spans
    }

    fn find_expr_spans<F>(&self, context: &LintContext, predicate: F) -> Vec<Span>
    where
        F: Fn(&Expression, &LintContext) -> bool,
    {
        use nu_protocol::ast::Expression;

        let mut matching_spans = Vec::new();
        self.flat_map(
            context.working_set,
            &|expr: &Expression| {
                if predicate(expr, context) {
                    vec![expr.span]
                } else {
                    vec![]
                }
            },
            &mut matching_spans,
        );
        matching_spans
    }
}
