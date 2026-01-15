use std::collections::HashSet;

use nu_protocol::{
    BlockId, Span, Type, VarId,
    ast::{Block, Expr, Expression, Pipeline, PipelineElement, Traverse},
};

use super::call::CallExt;
use crate::{ast::expression::ExpressionExt, context::LintContext};

const MAX_TYPE_INFERENCE_DEPTH: usize = 100;

fn find_transitively_called_functions_impl(
    block: &Block,
    context: &LintContext,
    available_functions: &HashSet<BlockId>,
    visited: &mut HashSet<BlockId>,
) -> HashSet<BlockId> {
    let mut result = HashSet::new();

    for callee_block_id in block.collect_user_function_call_block_ids(context) {
        if !available_functions.contains(&callee_block_id) {
            continue;
        }

        if !visited.insert(callee_block_id) {
            log::debug!("Cycle detected in function calls");
            continue;
        }

        result.insert(callee_block_id);

        let callee_block = context.working_set.get_block(callee_block_id);
        let transitive = find_transitively_called_functions_impl(
            callee_block,
            context,
            available_functions,
            visited,
        );
        result.extend(transitive);
    }

    result
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
    /// Collects all user function call block IDs in block. Returns the
    /// `BlockId` of each called custom command's body.
    fn collect_user_function_call_block_ids(&self, context: &LintContext) -> Vec<BlockId>;
    /// Finds all transitively called functions by `BlockId`. Example: main
    /// calls foo, foo calls bar - returns `BlockId`s of foo and bar
    fn find_transitively_called_functions(
        &self,
        context: &LintContext,
        available_functions: &HashSet<BlockId>,
    ) -> HashSet<BlockId>;
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

    /// Traverse block and all descendants with parent tracking.
    /// Calls the callback for each expression with its immediate parent.
    fn traverse_with_parent<'a, F>(
        &'a self,
        context: &'a LintContext,
        parent: Option<&'a Expression>,
        callback: &mut F,
    ) where
        F: FnMut(&'a Expression, Option<&'a Expression>);

    /// Recursively detect violations in all pipelines of this block and nested
    /// blocks. This is a common pattern used by many lint rules.
    ///
    /// The `check_pipeline` function is called for each pipeline and should
    /// return violations found in that pipeline. The function automatically
    /// recurses into closures, blocks, and subexpressions.
    fn detect_in_pipelines<T>(
        &self,
        context: &LintContext,
        check_pipeline: impl Fn(&Pipeline, &LintContext) -> Vec<T> + Copy,
    ) -> Vec<T>;

    /// Checks if this block is a pipeline ending with `columns` command and
    /// returns the span of the record expression (everything before `columns`).
    /// Example: `($record | columns)` returns span of `$record`
    fn find_columns_record_span(&self, context: &LintContext) -> Option<Span>;
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

    fn collect_user_function_call_block_ids(&self, context: &LintContext) -> Vec<BlockId> {
        let mut block_ids = Vec::new();

        self.flat_map(
            context.working_set,
            &|expr| {
                if let Expr::Call(call) = &expr.expr {
                    let decl = context.working_set.get_decl(call.decl_id);
                    decl.block_id().into_iter().collect()
                } else {
                    vec![]
                }
            },
            &mut block_ids,
        );

        block_ids
    }

    fn find_transitively_called_functions(
        &self,
        context: &LintContext,
        available_functions: &HashSet<BlockId>,
    ) -> HashSet<BlockId> {
        let mut visited: HashSet<BlockId> = HashSet::new();
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

    fn traverse_with_parent<'a, F>(
        &'a self,
        context: &'a LintContext,
        parent: Option<&'a Expression>,
        callback: &mut F,
    ) where
        F: FnMut(&'a Expression, Option<&'a Expression>),
    {
        use crate::ast::expression::ExpressionExt;

        // For each pipeline element, parent is the block/closure/subexpression
        // expression
        for pipeline in &self.pipelines {
            for element in &pipeline.elements {
                element.expr.traverse_with_parent(context, parent, callback);
            }
        }
    }

    fn detect_in_pipelines<T>(
        &self,
        context: &LintContext,
        check_pipeline: impl Fn(&Pipeline, &LintContext) -> Vec<T> + Copy,
    ) -> Vec<T> {
        let mut results: Vec<T> = self
            .pipelines
            .iter()
            .flat_map(|p| check_pipeline(p, context))
            .collect();

        // Recurse into nested blocks (closures, blocks, subexpressions)
        for pipeline in &self.pipelines {
            for element in &pipeline.elements {
                element.expr.flat_map(
                    context.working_set,
                    &|expr: &Expression| recurse_into_nested(expr, context, check_pipeline),
                    &mut results,
                );
            }
        }

        results
    }

    fn find_columns_record_span(&self, context: &LintContext) -> Option<Span> {
        let pipeline = self.pipelines.first()?;

        if pipeline.elements.len() < 2 {
            return None;
        }

        let last_elem = pipeline.elements.last()?;
        let Expr::Call(call) = &last_elem.expr.expr else {
            return None;
        };

        let decl = context.working_set.get_decl(call.decl_id);
        if decl.name() != "columns" {
            return None;
        }

        let elements_before_columns = &pipeline.elements[..pipeline.elements.len() - 1];
        if elements_before_columns.is_empty() {
            return None;
        }

        let start = elements_before_columns.first()?.expr.span.start;
        let end = elements_before_columns.last()?.expr.span.end;
        Some(Span::new(start, end))
    }
}

fn recurse_into_nested<T>(
    expr: &Expression,
    context: &LintContext,
    check_pipeline: impl Fn(&Pipeline, &LintContext) -> Vec<T> + Copy,
) -> Vec<T> {
    expr.extract_block_id()
        .map(|id| {
            context
                .working_set
                .get_block(id)
                .detect_in_pipelines(context, check_pipeline)
        })
        .unwrap_or_default()
}
