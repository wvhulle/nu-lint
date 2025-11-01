use nu_protocol::{BlockId, Span, ast::{Expr, Expression, Call, Pipeline, PipelineElement, Operator}};
use crate::{context::LintContext, lint::{Fix, Replacement, RuleViolation}};

/// Common utilities for AST analysis across rules
pub struct AstUtils;

impl AstUtils {
    /// Check if two expressions refer to the same variable
    pub fn expressions_refer_to_same_variable(
        expr1: &Expression,
        expr2: &Expression,
        context: &LintContext,
    ) -> bool {
        let text1 = &context.source[expr1.span.start..expr1.span.end];
        let text2 = &context.source[expr2.span.start..expr2.span.end];
        text1 == text2
    }

    /// Extract variable name from an expression
    pub fn extract_variable_name(expr: &Expression, context: &LintContext) -> Option<String> {
        match &expr.expr {
            Expr::Var(var_id) => {
                let var = context.working_set.get_variable(*var_id);
                Some(context.source[var.declaration_span.start..var.declaration_span.end].to_string())
            }
            Expr::VarDecl(var_id) => {
                let var = context.working_set.get_variable(*var_id);
                Some(context.source[var.declaration_span.start..var.declaration_span.end].to_string())
            }
            _ => {
                let text = &context.source[expr.span.start..expr.span.end];
                Some(text.to_string())
            }
        }
    }

    /// Get the declaration name from a call expression
    pub fn get_call_name(call: &Call, context: &LintContext) -> String {
        context.working_set.get_decl(call.decl_id).name().to_string()
    }

    /// Check if an expression is a call to a specific command
    pub fn is_call_to_command(expr: &Expression, command_name: &str, context: &LintContext) -> bool {
        match &expr.expr {
            Expr::Call(call) => Self::get_call_name(call, context) == command_name,
            _ => false,
        }
    }

    /// Extract the first positional argument from a call as an expression
    pub fn get_first_positional_arg(call: &Call) -> Option<&Expression> {
        call.arguments.first().and_then(|arg| match arg {
            nu_protocol::ast::Argument::Positional(expr) |
            nu_protocol::ast::Argument::Unknown(expr) => Some(expr),
            _ => None,
        })
    }

    /// Extract the nth positional argument from a call as an expression
    pub fn get_positional_arg(call: &Call, index: usize) -> Option<&Expression> {
        call.arguments.get(index).and_then(|arg| match arg {
            nu_protocol::ast::Argument::Positional(expr) |
            nu_protocol::ast::Argument::Unknown(expr) => Some(expr),
            _ => None,
        })
    }

    /// Check if a variable/expression refers to a specific variable name
    pub fn refers_to_variable(
        expr: &Expression,
        context: &LintContext,
        var_name: &str,
    ) -> bool {
        match &expr.expr {
            Expr::Var(var_id) => {
                let var = context.working_set.get_variable(*var_id);
                let actual_name = &context.source[var.declaration_span.start..var.declaration_span.end];
                actual_name == var_name
            }
            Expr::FullCellPath(cell_path) => {
                Self::refers_to_variable(&cell_path.head, context, var_name)
            }
            _ => false,
        }
    }

    /// Extract block ID from various expression types that contain blocks
    pub fn extract_block_id(expr: &Expression) -> Option<BlockId> {
        match &expr.expr {
            Expr::Block(id) | Expr::Closure(id) | Expr::Subexpression(id) => Some(*id),
            _ => None,
        }
    }

    /// Check if a pipeline has a specific number of elements
    pub fn pipeline_has_element_count(pipeline: &Pipeline, count: usize) -> bool {
        pipeline.elements.len() == count
    }

    /// Get the first element of a pipeline if it exists
    pub fn get_first_pipeline_element(pipeline: &Pipeline) -> Option<&PipelineElement> {
        pipeline.elements.first()
    }

    /// Get the last element of a pipeline if it exists
    pub fn get_last_pipeline_element(pipeline: &Pipeline) -> Option<&PipelineElement> {
        pipeline.elements.last()
    }

    /// Check if a block contains side effects (commands that modify state)
    pub fn has_side_effects(block_id: BlockId, context: &LintContext) -> bool {
        let block = context.working_set.get_block(block_id);

        block
            .pipelines
            .iter()
            .flat_map(|p| &p.elements)
            .any(|elem| Self::is_side_effect_expression(&elem.expr, context))
    }

    /// Check if an expression represents a side effect
    pub fn is_side_effect_expression(expr: &Expression, context: &LintContext) -> bool {
        match &expr.expr {
            Expr::Call(call) => {
                let decl_name = Self::get_call_name(call, context);
                matches!(
                    decl_name.as_str(),
                    "print" | "save" | "download" | "exit" | "mut" | "cd" | "source" | "use"
                )
            }
            Expr::BinaryOp(_, op, _) => {
                matches!(
                    op.expr,
                    Expr::Operator(Operator::Assignment(_))
                )
            }
            _ => false,
        }
    }

    /// Check if an expression is an assignment operation
    pub fn is_assignment(expr: &Expression) -> bool {
        match &expr.expr {
            Expr::BinaryOp(_, op, _) => {
                matches!(op.expr, Expr::Operator(Operator::Assignment(_)))
            }
            _ => false,
        }
    }

    /// Check if an expression is a specific type of binary operation
    pub fn is_binary_op_with_operator(expr: &Expression, operator: Operator) -> bool {
        match &expr.expr {
            Expr::BinaryOp(_, op, _) => {
                matches!(op.expr, Expr::Operator(op_val) if op_val == operator)
            }
            _ => false,
        }
    }

    /// Extract text from a span
    pub fn span_text<'a>(span: nu_protocol::Span, context: &'a LintContext) -> &'a str {
        &context.source[span.start..span.end]
    }

    /// Check if an expression is an empty list
    pub fn is_empty_list(expr: &Expression) -> bool {
        match &expr.expr {
            Expr::List(items) => items.is_empty(),
            Expr::FullCellPath(cell_path) => {
                matches!(&cell_path.head.expr, Expr::List(items) if items.is_empty())
            }
            _ => false,
        }
    }

    /// Check if a block contains only an empty list
    pub fn is_empty_list_block(block_id: BlockId, context: &LintContext) -> bool {
        let block = context.working_set.get_block(block_id);

        block.pipelines.first()
            .and_then(|pipeline| pipeline.elements.first())
            .map(|elem| Self::is_empty_list(&elem.expr))
            .unwrap_or(false)
    }
}

/// Trait for pattern matching in AST expressions
pub trait PatternMatcher {
    fn matches(&self, expr: &Expression, context: &LintContext) -> bool;
}

/// Pattern for matching specific command calls
pub struct CommandCallPattern {
    pub command_name: String,
}

impl CommandCallPattern {
    pub fn new(command_name: impl Into<String>) -> Self {
        Self {
            command_name: command_name.into(),
        }
    }
}

impl PatternMatcher for CommandCallPattern {
    fn matches(&self, expr: &Expression, context: &LintContext) -> bool {
        AstUtils::is_call_to_command(expr, &self.command_name, context)
    }
}

/// Pattern for matching assignment operations
pub struct AssignmentPattern;

impl PatternMatcher for AssignmentPattern {
    fn matches(&self, expr: &Expression, _context: &LintContext) -> bool {
        AstUtils::is_assignment(expr)
    }
}

/// Pattern for matching variable references
pub struct VariablePattern {
    pub variable_name: String,
}

impl VariablePattern {
    pub fn new(variable_name: impl Into<String>) -> Self {
        Self {
            variable_name: variable_name.into(),
        }
    }
}

impl PatternMatcher for VariablePattern {
    fn matches(&self, expr: &Expression, context: &LintContext) -> bool {
        AstUtils::refers_to_variable(expr, context, &self.variable_name)
    }
}

/// Utility for extracting loop variables from common control structures
pub struct LoopVariableExtractor;

impl LoopVariableExtractor {
    /// Extract loop variable name from 'each' command
    pub fn from_each_call(call: &Call, context: &LintContext) -> Option<String> {
        let first_arg = AstUtils::get_first_positional_arg(call)?;
        let block_id = AstUtils::extract_block_id(first_arg)?;

        let block = context.working_set.get_block(block_id);
        let var_id = block.signature.required_positional.first()?.var_id?;

        let var = context.working_set.get_variable(var_id);
        Some(AstUtils::span_text(var.declaration_span, context).to_string())
    }

    /// Extract loop variable name from 'for' command
    pub fn from_for_call(call: &Call, context: &LintContext) -> Option<String> {
        let var_arg = AstUtils::get_first_positional_arg(call)?;
        AstUtils::extract_variable_name(var_arg, context)
    }
}


/// Utilities for naming convention validation and fixes
pub struct NamingUtils;

impl NamingUtils {
    /// Create a naming convention violation with fix
    pub fn create_naming_violation(
        rule_id: &'static str,
        item_type: &str,
        current_name: &str,
        suggested_name: String,
        name_span: Span,
    ) -> RuleViolation {
        let fix = Fix {
            description: format!("Rename {item_type} '{current_name}' to '{suggested_name}'").into(),
            replacements: vec![Replacement {
                span: name_span,
                new_text: suggested_name.clone().into(),
            }],
        };

        RuleViolation::new_dynamic(
            rule_id,
            format!("{item_type} '{current_name}' should follow naming convention"),
            name_span,
        )
        .with_suggestion_dynamic(format!("Consider renaming to: {suggested_name}"))
        .with_fix(fix)
    }

    /// Check if a name is valid kebab-case
    pub fn is_valid_kebab_case(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        if name.len() == 1 {
            return name.chars().all(|c| c.is_ascii_lowercase());
        }

        name.chars().enumerate().all(|(i, c)| {
            match c {
                'a'..='z' | '0'..='9' => true,
                '-' => {
                    if i == 0 {
                        return false;
                    }
                    name.chars().nth(i + 1) != Some('-')
                }
                _ => false,
            }
        }) && name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
    }

    /// Check if a name is valid snake_case
    pub fn is_valid_snake_case(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        if name.len() == 1 {
            return name.chars().all(|c| c.is_ascii_lowercase() || c == '_');
        }

        let first_char = name.chars().next().unwrap();
        if !first_char.is_ascii_lowercase() && first_char != '_' {
            return false;
        }

        let chars: Vec<char> = name.chars().collect();
        chars.windows(2).all(|w| {
            let (current, next) = (w[0], w[1]);
            let valid_char = matches!(current, 'a'..='z' | '0'..='9' | '_');
            let no_double_underscore = !(current == '_' && next == '_');
            valid_char && no_double_underscore
        }) && matches!(chars.last(), Some('a'..='z' | '0'..='9' | '_'))
    }
}

/// Utilities for checking declaration commands (def, let, mut, etc.)
pub struct DeclarationUtils;

impl DeclarationUtils {
    /// Check if a command is a def declaration
    pub fn is_def_command(decl_name: &str) -> bool {
        matches!(decl_name, "def" | "export def")
    }

    /// Check if a command is a variable declaration, returns (is_declaration, is_mutable)
    pub fn is_var_declaration(decl_name: &str) -> Option<bool> {
        match decl_name {
            "let" => Some(false),
            "mut" => Some(true),
            _ => None,
        }
    }

    /// Extract the first argument (name) from a declaration call
    pub fn extract_declaration_name(call: &Call, context: &LintContext) -> Option<(String, Span)> {
        let name_arg = AstUtils::get_first_positional_arg(call)?;
        let name = context.source.get(name_arg.span.start..name_arg.span.end)?;
        Some((name.to_string(), name_arg.span))
    }

    /// Extract function definition information from a call
    pub fn extract_function_definition(call: &Call, context: &LintContext) -> Option<(BlockId, String)> {
        let decl_name = AstUtils::get_call_name(call, context);
        if !Self::is_def_command(&decl_name) {
            return None;
        }

        // First argument is the function name
        let name_arg = AstUtils::get_first_positional_arg(call)?;
        let name = AstUtils::span_text(name_arg.span, context);

        // Third argument is the function body block (can be Block or Closure)
        let body_expr = AstUtils::get_positional_arg(call, 2)?;
        let block_id = AstUtils::extract_block_id(body_expr)?;

        Some((block_id, name.to_string()))
    }

    /// Extract variable declaration info (var_id, name, span) from let/mut calls
    pub fn extract_variable_declaration(call: &Call, context: &LintContext) -> Option<(nu_protocol::VarId, String, Span)> {
        let decl_name = AstUtils::get_call_name(call, context);
        if !matches!(decl_name.as_str(), "let" | "mut") {
            return None;
        }

        let var_arg = AstUtils::get_first_positional_arg(call)?;

        if let Expr::VarDecl(var_id) = &var_arg.expr {
            let var_name = AstUtils::span_text(var_arg.span, context);
            Some((*var_id, var_name.to_string(), var_arg.span))
        } else {
            None
        }
    }
}

/// Utilities for working with assignments and variable usage
pub struct VariableUtils;

impl VariableUtils {
    /// Extract variable ID from assignment expressions
    pub fn extract_assigned_variable(expr: &Expression) -> Option<nu_protocol::VarId> {
        let Expr::BinaryOp(lhs, _op, _rhs) = &expr.expr else {
            return None;
        };

        if !AstUtils::is_assignment(expr) {
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

    /// Check if a cell path accesses a specific field
    pub fn accesses_field(path_tail: &[nu_protocol::ast::PathMember], field_name: &str) -> bool {
        path_tail.iter().any(|path_member| {
            matches!(
                path_member,
                nu_protocol::ast::PathMember::String { val, .. } if val == field_name
            )
        })
    }

    /// Extract variable accesses with field access
    pub fn extract_field_access(expr: &Expression, field_name: &str) -> Option<(nu_protocol::VarId, Span)> {
        if let Expr::FullCellPath(cell_path) = &expr.expr
            && let Expr::Var(var_id) = &cell_path.head.expr
            && Self::accesses_field(&cell_path.tail, field_name)
        {
            Some((*var_id, expr.span))
        } else {
            None
        }
    }
}

/// Utilities for working with blocks and nested structures
pub struct BlockUtils;

impl BlockUtils {
    /// Check if a span is contained within a block
    pub fn span_in_block(span: Span, block_id: BlockId, context: &LintContext) -> bool {
        let block = context.working_set.get_block(block_id);
        if let Some(block_span) = block.span {
            return span.start >= block_span.start && span.end <= block_span.end;
        }
        false
    }

    /// Find which function contains a given span (returns the most specific one)
    pub fn find_containing_function(
        span: Span,
        functions: &std::collections::HashMap<BlockId, String>,
        context: &LintContext,
    ) -> Option<String> {
        functions
            .iter()
            .filter(|(block_id, _)| Self::span_in_block(span, **block_id, context))
            .min_by_key(|(block_id, _)| {
                let block = context.working_set.get_block(**block_id);
                block.span.map_or(usize::MAX, |s| s.end - s.start)
            })
            .map(|(_, name)| name.clone())
    }

    /// Extract all nested block IDs from an expression
    pub fn extract_nested_blocks(expr: &Expression, context: &LintContext) -> Vec<BlockId> {
        use nu_protocol::ast::Traverse;

        let mut blocks = Vec::new();
        expr.flat_map(
            context.working_set,
            &|inner_expr| {
                AstUtils::extract_block_id(inner_expr).into_iter().collect()
            },
            &mut blocks,
        );
        blocks
    }
}