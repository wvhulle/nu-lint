use nu_protocol::{
    BlockId, Span, VarId,
    ast::{Call, Expr, Expression, Operator, PathMember, Pipeline, PipelineElement},
};

use crate::context::LintContext;

/// Trait to extend Expression with utility methods
pub trait ExpressionExt {
    /// Check if this expression refers to the same variable as another
    fn refers_to_same_variable(&self, other: &Expression, context: &LintContext) -> bool;

    /// Extract variable name from this expression
    fn extract_variable_name(&self, context: &LintContext) -> Option<String>;

    /// Check if this expression refers to a specific variable by name
    fn refers_to_variable(&self, context: &LintContext, var_name: &str) -> bool;

    /// Check if this expression is an assignment
    fn is_assignment(&self) -> bool;

    /// Check if this expression is an empty list
    fn is_empty_list(&self) -> bool;

    /// Extract block ID if this expression is a block or closure
    fn extract_block_id(&self) -> Option<BlockId>;

    /// Check if this expression is likely pure (no side effects).
    /// Returns true only for expressions that are definitively pure (variable
    /// references, literals, cell paths). For anything else (including most
    /// command calls), conservatively returns false.
    fn is_likely_pure(&self) -> bool;

    /// Get the text span content for this expression
    fn span_text<'a>(&self, context: &'a LintContext) -> &'a str;

    /// Check if this expression is a call to a specific command
    fn is_call_to(&self, command_name: &str, context: &LintContext) -> bool;

    /// Extract variable ID from assignment expressions
    fn extract_assigned_variable(&self) -> Option<nu_protocol::VarId>;

    /// Extract variable accesses with field access
    fn extract_field_access(&self, field_name: &str) -> Option<(nu_protocol::VarId, Span)>;

    /// Check if this expression is an external command call
    fn is_external_call(&self) -> bool;

    /// Extract external command name if this is an external call
    fn extract_external_command_name(&self, context: &LintContext) -> Option<String>;

    /// Check if this expression contains a call to a specific command
    /// (recursive)
    fn contains_call_to(&self, command_name: &str, context: &LintContext) -> bool;

    /// Check if this expression contains any variable references (recursive)
    fn contains_variables(&self, context: &LintContext) -> bool;

    /// Extract the variable being compared in an equality comparison expression
    /// Returns the variable name if this is a pattern like `$var == value` or
    /// `$var != value`
    fn extract_compared_variable(&self, context: &LintContext) -> Option<String>;

    /// Extract the comparison value from an equality comparison expression
    /// Returns the right-hand side value if this is `$var == value`
    fn extract_comparison_value(&self, context: &LintContext) -> Option<String>;
}

impl ExpressionExt for Expression {
    fn refers_to_same_variable(&self, other: &Expression, context: &LintContext) -> bool {
        let text1 = self.span_text(context);
        let text2 = other.span_text(context);
        text1 == text2
    }

    fn extract_variable_name(&self, context: &LintContext) -> Option<String> {
        match &self.expr {
            Expr::Var(var_id) | Expr::VarDecl(var_id) => {
                let var = context.working_set.get_variable(*var_id);
                Some(
                    context.source[var.declaration_span.start..var.declaration_span.end]
                        .to_string(),
                )
            }
            Expr::FullCellPath(cell_path) => {
                // For $x.field, extract just the variable name
                cell_path.head.extract_variable_name(context)
            }
            _ => None,
        }
    }

    fn refers_to_variable(&self, context: &LintContext, var_name: &str) -> bool {
        if let Some(name) = self.extract_variable_name(context) {
            name == var_name
        } else {
            false
        }
    }

    fn is_assignment(&self) -> bool {
        matches!(
            &self.expr,
            Expr::BinaryOp(_, op, _) if matches!(
                op.expr,
                Expr::Operator(Operator::Assignment(_))
            )
        )
    }

    fn is_empty_list(&self) -> bool {
        match &self.expr {
            Expr::List(list) => list.is_empty(),
            Expr::FullCellPath(cell_path) => cell_path.head.is_empty_list(),
            _ => false,
        }
    }

    fn extract_block_id(&self) -> Option<BlockId> {
        match &self.expr {
            Expr::Block(block_id) | Expr::Closure(block_id) | Expr::Subexpression(block_id) => {
                Some(*block_id)
            }
            _ => None,
        }
    }

    fn is_likely_pure(&self) -> bool {
        match &self.expr {
            // These are definitively pure
            Expr::Bool(_)
            | Expr::Int(_)
            | Expr::Float(_)
            | Expr::Binary(_)
            | Expr::String(_)
            | Expr::RawString(_)
            | Expr::Filepath(_, _)
            | Expr::Directory(_, _)
            | Expr::GlobPattern(_, _)
            | Expr::List(_)
            | Expr::Record(_)
            | Expr::Table(_)
            | Expr::Keyword(_)
            | Expr::Nothing
            | Expr::ValueWithUnit(_)
            | Expr::DateTime(_)
            | Expr::Range(_)
            | Expr::Var(_)
            | Expr::VarDecl(_)
            | Expr::FullCellPath(_) => true,

            // Operators can be pure if both sides are pure
            Expr::BinaryOp(left, op, right) => {
                // Assignment is not pure
                if matches!(op.expr, Expr::Operator(Operator::Assignment(_))) {
                    return false;
                }
                left.is_likely_pure() && right.is_likely_pure()
            }

            Expr::UnaryNot(inner) => inner.is_likely_pure(),

            // Conservatively assume calls, external calls, closures etc have side
            // effects
            _ => false,
        }
    }

    fn span_text<'a>(&self, context: &'a LintContext) -> &'a str {
        &context.source[self.span.start..self.span.end]
    }

    fn is_call_to(&self, command_name: &str, context: &LintContext) -> bool {
        match &self.expr {
            Expr::Call(call) => call.is_call_to_command(command_name, context),
            _ => false,
        }
    }

    fn extract_assigned_variable(&self) -> Option<nu_protocol::VarId> {
        let Expr::BinaryOp(lhs, _op, _rhs) = &self.expr else {
            return None;
        };

        if !self.is_assignment() {
            return None;
        }

        match &lhs.expr {
            Expr::Var(var_id) => Some(*var_id),
            Expr::FullCellPath(cell_path) => {
                if let Expr::Var(var_id) = &cell_path.head.expr {
                    Some(*var_id)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn extract_field_access(&self, field_name: &str) -> Option<(nu_protocol::VarId, Span)> {
        if let Expr::FullCellPath(cell_path) = &self.expr
            && let Expr::Var(var_id) = &cell_path.head.expr
            && VariableUtils::accesses_field(&cell_path.tail, field_name)
        {
            Some((*var_id, self.span))
        } else {
            None
        }
    }

    fn is_external_call(&self) -> bool {
        matches!(&self.expr, Expr::ExternalCall(_, _))
    }

    fn extract_external_command_name(&self, context: &LintContext) -> Option<String> {
        if let Expr::ExternalCall(head, _args) = &self.expr {
            Some(head.span.text(context).to_string())
        } else {
            None
        }
    }

    fn contains_call_to(&self, command_name: &str, context: &LintContext) -> bool {
        use nu_protocol::ast::Traverse;

        let mut results = Vec::new();
        self.flat_map(
            context.working_set,
            &|inner_expr| {
                if inner_expr.is_call_to(command_name, context) {
                    vec![true]
                } else {
                    vec![]
                }
            },
            &mut results,
        );
        !results.is_empty()
    }

    fn contains_variables(&self, context: &LintContext) -> bool {
        match &self.expr {
            // Direct variable reference
            Expr::Var(_) | Expr::VarDecl(_) => true,

            // Cell paths (like $x.field)
            Expr::FullCellPath(path) => path.head.contains_variables(context),

            // Subexpressions, blocks, closures
            Expr::Subexpression(block_id) | Expr::Block(block_id) | Expr::Closure(block_id) => {
                block_id.contains_variables(context)
            }

            // Binary operators
            Expr::BinaryOp(left, _, right) => {
                left.contains_variables(context) || right.contains_variables(context)
            }

            // Unary operators
            Expr::UnaryNot(inner) => inner.contains_variables(context),

            // Lists and records might contain variables
            Expr::List(items) => items.iter().any(|item| match item {
                nu_protocol::ast::ListItem::Item(expr)
                | nu_protocol::ast::ListItem::Spread(_, expr) => expr.contains_variables(context),
            }),

            Expr::Record(fields) => fields.iter().any(|field| match field {
                nu_protocol::ast::RecordItem::Pair(key, val) => {
                    key.contains_variables(context) || val.contains_variables(context)
                }
                nu_protocol::ast::RecordItem::Spread(_, expr) => expr.contains_variables(context),
            }),

            // Calls might have variables in arguments
            Expr::Call(call) => call.arguments.iter().any(|arg| match arg {
                nu_protocol::ast::Argument::Positional(expr)
                | nu_protocol::ast::Argument::Unknown(expr)
                | nu_protocol::ast::Argument::Named((_, _, Some(expr))) => {
                    expr.contains_variables(context)
                }
                _ => false,
            }),

            // Everything else (literals, nothing, etc.) doesn't contain variables
            _ => false,
        }
    }

    fn extract_compared_variable(&self, context: &LintContext) -> Option<String> {
        let Expr::BinaryOp(left, op, _right) = &self.expr else {
            return None;
        };

        // Check if it's an equality comparison
        let Expr::Operator(Operator::Comparison(
            nu_protocol::ast::Comparison::Equal | nu_protocol::ast::Comparison::NotEqual,
        )) = &op.expr
        else {
            return None;
        };

        // Try to extract variable name from left side
        if let Some(var_name) = left.extract_variable_name(context) {
            return Some(var_name);
        }

        // Fallback: if the left side looks like a variable reference (FullCellPath),
        // extract the text even if the variable isn't properly declared
        if let Expr::FullCellPath(cell_path) = &left.expr {
            Some(cell_path.head.span.text(context).to_string())
        } else {
            None
        }
    }

    fn extract_comparison_value(&self, context: &LintContext) -> Option<String> {
        let Expr::BinaryOp(_left, _op, right) = &self.expr else {
            return None;
        };

        Some(right.span_text(context).to_string())
    }
}

/// Trait to extend Call with utility methods
pub trait CallExt {
    /// Get the declaration name from this call
    fn get_call_name(&self, context: &LintContext) -> String;

    /// Check if this call is to a specific command
    fn is_call_to_command(&self, command_name: &str, context: &LintContext) -> bool;

    /// Get the first positional argument from this call
    fn get_first_positional_arg(&self) -> Option<&Expression>;

    /// Get a positional argument by index
    fn get_positional_arg(&self, index: usize) -> Option<&Expression>;

    /// Extract loop variable name from 'each' command
    #[must_use]
    fn loop_var_from_each(&self, context: &LintContext) -> Option<String>;

    /// Extract loop variable name from 'for' command
    #[must_use]
    fn loop_var_from_for(&self, context: &LintContext) -> Option<String>;

    /// Extract the first argument (name) from a declaration call
    #[must_use]
    fn extract_declaration_name(&self, context: &LintContext) -> Option<(String, Span)>;

    /// Extract function definition information from this call
    #[must_use]
    fn extract_function_definition(&self, context: &LintContext) -> Option<(BlockId, String)>;

    /// Extract variable declaration info (`var_id`, name, span) from let/mut
    /// calls
    #[must_use]
    fn extract_variable_declaration(
        &self,
        context: &LintContext,
    ) -> Option<(nu_protocol::VarId, String, Span)>;

    /// Get the else branch from an if call
    ///
    /// Returns `(is_else_if, nested_call_or_block_expr)` where:
    /// - `is_else_if` = true if it's another if call (else if)
    /// - `is_else_if` = false if it's a final else block
    ///
    /// Returns `None` if there's no else branch
    fn get_else_branch(&self) -> Option<(bool, &Expression)>;
}

impl CallExt for Call {
    fn get_call_name(&self, context: &LintContext) -> String {
        context
            .working_set
            .get_decl(self.decl_id)
            .name()
            .to_string()
    }

    fn is_call_to_command(&self, command_name: &str, context: &LintContext) -> bool {
        self.get_call_name(context) == command_name
    }

    fn get_first_positional_arg(&self) -> Option<&Expression> {
        self.arguments.first().and_then(|arg| match arg {
            nu_protocol::ast::Argument::Positional(expr)
            | nu_protocol::ast::Argument::Unknown(expr) => Some(expr),
            _ => None,
        })
    }

    fn get_positional_arg(&self, index: usize) -> Option<&Expression> {
        self.arguments.get(index).and_then(|arg| match arg {
            nu_protocol::ast::Argument::Positional(expr)
            | nu_protocol::ast::Argument::Unknown(expr) => Some(expr),
            _ => None,
        })
    }

    fn loop_var_from_each(&self, context: &LintContext) -> Option<String> {
        let first_arg = self.get_first_positional_arg()?;
        let block_id = first_arg.extract_block_id()?;

        let block = context.working_set.get_block(block_id);
        let var_id = block.signature.required_positional.first()?.var_id?;

        let var = context.working_set.get_variable(var_id);
        Some(var.declaration_span.text(context).to_string())
    }

    fn loop_var_from_for(&self, context: &LintContext) -> Option<String> {
        let var_arg = self.get_first_positional_arg()?;
        var_arg.extract_variable_name(context)
    }

    fn extract_declaration_name(&self, context: &LintContext) -> Option<(String, Span)> {
        let name_arg = self.get_first_positional_arg()?;
        let name = context.source.get(name_arg.span.start..name_arg.span.end)?;
        Some((name.to_string(), name_arg.span))
    }

    fn extract_function_definition(&self, context: &LintContext) -> Option<(BlockId, String)> {
        let decl_name = self.get_call_name(context);
        if !matches!(decl_name.as_str(), "def" | "export def") {
            return None;
        }

        // First argument is the function name
        let name_arg = self.get_first_positional_arg()?;
        let name = name_arg.span.text(context);

        // Third argument is the function body block (can be Block or Closure)
        let body_expr = self.get_positional_arg(2)?;
        let block_id = body_expr.extract_block_id()?;

        Some((block_id, name.to_string()))
    }

    fn extract_variable_declaration(
        &self,
        context: &LintContext,
    ) -> Option<(nu_protocol::VarId, String, Span)> {
        let decl_name = self.get_call_name(context);
        if !matches!(decl_name.as_str(), "let" | "mut") {
            return None;
        }

        let var_arg = self.get_first_positional_arg()?;

        if let Expr::VarDecl(var_id) = &var_arg.expr {
            let var_name = var_arg.span.text(context);
            Some((*var_id, var_name.to_string(), var_arg.span))
        } else {
            None
        }
    }

    fn get_else_branch(&self) -> Option<(bool, &Expression)> {
        // Get the else argument (3rd positional argument for if command)
        let else_arg = self.get_positional_arg(2)?;

        // The else argument can be either:
        // 1. A Block (final else { ... })
        // 2. A Keyword containing either:
        //    - An if Call (else if ...)
        //    - A Block (final else { ... } wrapped in Keyword)
        match &else_arg.expr {
            Expr::Keyword(keyword) => {
                // Check what the Keyword contains
                match &keyword.expr.expr {
                    Expr::Call(_) => {
                        // This is an else-if chain
                        Some((true, &keyword.expr))
                    }
                    Expr::Block(_) => {
                        // This is a final else block wrapped in a Keyword
                        Some((false, &keyword.expr))
                    }
                    _ => None,
                }
            }
            Expr::Block(_) => {
                // This is a final else block (not wrapped in Keyword)
                Some((false, else_arg))
            }
            _ => None,
        }
    }
}

/// Trait to extend `BlockId` with utility methods
pub trait BlockExt {
    /// Check if this block likely has side effects.
    /// This is a conservative check: returns true if the block contains any
    /// expressions that might have side effects (which includes most command
    /// calls). Only returns false if all expressions are definitively pure
    /// (literals, variables, pure operators).
    fn has_side_effects(&self, context: &LintContext) -> bool;

    /// Check if this block contains only an empty list
    fn is_empty_list_block(&self, context: &LintContext) -> bool;

    /// Check if a span is contained within this block
    #[must_use]
    fn contains_span(&self, span: Span, context: &LintContext) -> bool;

    /// Get all pipeline elements from this block
    fn all_elements<'a>(&self, context: &'a LintContext) -> Vec<&'a PipelineElement>;

    /// Check if this block contains a call to a specific command
    fn contains_call_to(&self, command_name: &str, context: &LintContext) -> bool;

    /// Check if any element in this block matches a predicate
    fn any_element<F>(&self, context: &LintContext, predicate: F) -> bool
    where
        F: Fn(&PipelineElement) -> bool;

    /// Check if this block contains any variable references
    fn contains_variables(&self, context: &LintContext) -> bool;

    /// Check if this is a single-pipeline block containing a call to a specific command
    fn contains_call_in_single_pipeline(&self, command_name: &str, context: &LintContext) -> bool;
}

impl BlockExt for BlockId {
    fn has_side_effects(&self, context: &LintContext) -> bool {
        let block = context.working_set.get_block(*self);

        block
            .pipelines
            .iter()
            .flat_map(|p| &p.elements)
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

    fn contains_call_to(&self, command_name: &str, context: &LintContext) -> bool {
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
        let block = context.working_set.get_block(*self);
        block
            .pipelines
            .iter()
            .flat_map(|p| &p.elements)
            .any(|elem| elem.expr.contains_variables(context))
    }

    fn contains_call_in_single_pipeline(&self, command_name: &str, context: &LintContext) -> bool {
        let block = context.working_set.get_block(*self);
        block.pipelines.len() == 1 && block.pipelines[0].contains_call_to(command_name, context)
    }
}

/// Trait to extend Span with utility methods
pub trait SpanExt {
    /// Get the text content for this span
    #[must_use]
    fn text<'a>(&self, context: &'a LintContext) -> &'a str;

    /// Find which function contains a given span (returns the most specific
    /// one)
    #[must_use]
    fn find_containing_function(
        &self,
        functions: &std::collections::HashMap<BlockId, String>,
        context: &LintContext,
    ) -> Option<String>;
}

impl SpanExt for Span {
    fn text<'a>(&self, context: &'a LintContext) -> &'a str {
        &context.source[self.start..self.end]
    }

    fn find_containing_function(
        &self,
        functions: &std::collections::HashMap<BlockId, String>,
        context: &LintContext,
    ) -> Option<String> {
        functions
            .iter()
            .filter(|(block_id, _)| block_id.contains_span(*self, context))
            .min_by_key(|(block_id, _)| {
                let block = context.working_set.get_block(**block_id);
                block.span.map_or(usize::MAX, |s| s.end - s.start)
            })
            .map(|(_, name)| name.clone())
    }
}

/// Utilities for working with assignments and variable usage
pub struct VariableUtils;

impl VariableUtils {
    /// Check if a cell path accesses a specific field
    #[must_use]
    pub fn accesses_field(path_tail: &[PathMember], field_name: &str) -> bool {
        path_tail.iter().any(|path_member| {
            matches!(
                path_member,
                PathMember::String { val, .. } if val == field_name
            )
        })
    }
}

/// Trait to extend Pipeline with utility methods
pub trait PipelineExt {
    /// Check if this pipeline contains a call to a specific command
    fn contains_call_to(&self, command_name: &str, context: &LintContext) -> bool;

    /// Check if this pipeline contains an indexed access call (get/skip with numeric index)
    fn contains_indexed_access(&self, context: &LintContext) -> bool;

    /// Check if a variable is used anywhere in this pipeline
    fn variable_is_used(&self, var_id: VarId) -> bool;

    /// Check if a variable is the first element being piped to something
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
