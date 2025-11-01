use nu_protocol::{
    BlockId, Span,
    ast::{Call, Expr, Expression, Operator, Pipeline, PipelineElement},
};

use crate::{
    context::LintContext,
    violation::{Fix, Replacement, RuleViolation},
};

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

    /// Check if this expression has side effects
    fn has_side_effects(&self, context: &LintContext) -> bool;

    /// Check if this expression represents a side effect
    fn is_side_effect_expression(&self, context: &LintContext) -> bool;

    /// Get the text span content for this expression
    fn span_text<'a>(&self, context: &'a LintContext) -> &'a str;
}

impl ExpressionExt for Expression {
    fn refers_to_same_variable(&self, other: &Expression, context: &LintContext) -> bool {
        let text1 = self.span_text(context);
        let text2 = other.span_text(context);
        text1 == text2
    }

    fn extract_variable_name(&self, context: &LintContext) -> Option<String> {
        match &self.expr {
            Expr::Var(var_id) => {
                let var = context.working_set.get_variable(*var_id);
                Some(
                    context.source[var.declaration_span.start..var.declaration_span.end]
                        .to_string(),
                )
            }
            Expr::VarDecl(var_id) => {
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

    fn has_side_effects(&self, context: &LintContext) -> bool {
        match &self.expr {
            Expr::Call(call) => {
                let decl_name = call.get_call_name(context);
                !matches!(
                    decl_name.as_str(),
                    "get" | "select" | "where" | "length" | "type"
                )
            }
            _ => false,
        }
    }

    fn is_side_effect_expression(&self, context: &LintContext) -> bool {
        match &self.expr {
            Expr::Call(call) => {
                let decl_name = call.get_call_name(context);
                matches!(
                    decl_name.as_str(),
                    "print" | "save" | "download" | "exit" | "mut" | "cd" | "source" | "use"
                )
            }
            Expr::BinaryOp(_, op, _) => {
                matches!(op.expr, Expr::Operator(Operator::Assignment(_)))
            }
            _ => false,
        }
    }

    fn span_text<'a>(&self, context: &'a LintContext) -> &'a str {
        &context.source[self.span.start..self.span.end]
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
}

/// Trait to extend Pipeline with utility methods
pub trait PipelineExt {
    /// Check if this pipeline has a specific number of elements
    fn has_element_count(&self, count: usize) -> bool;

    /// Get the first element of this pipeline if it exists
    fn get_first_element(&self) -> Option<&PipelineElement>;

    /// Get the last element of this pipeline if it exists
    fn get_last_element(&self) -> Option<&PipelineElement>;
}

impl PipelineExt for Pipeline {
    fn has_element_count(&self, count: usize) -> bool {
        self.elements.len() == count
    }

    fn get_first_element(&self) -> Option<&PipelineElement> {
        self.elements.first()
    }

    fn get_last_element(&self) -> Option<&PipelineElement> {
        self.elements.last()
    }
}

/// Trait to extend BlockId with utility methods
pub trait BlockExt {
    /// Check if this block contains side effects (commands that modify state)
    fn has_side_effects(&self, context: &LintContext) -> bool;

    /// Check if this block contains only an empty list
    fn is_empty_list_block(&self, context: &LintContext) -> bool;
}

impl BlockExt for BlockId {
    fn has_side_effects(&self, context: &LintContext) -> bool {
        let block = context.working_set.get_block(*self);

        block
            .pipelines
            .iter()
            .flat_map(|p| &p.elements)
            .any(|elem| elem.expr.is_side_effect_expression(context))
    }

    fn is_empty_list_block(&self, context: &LintContext) -> bool {
        let block = context.working_set.get_block(*self);

        block
            .pipelines
            .first()
            .and_then(|pipeline| pipeline.get_first_element())
            .map(|elem| elem.expr.is_empty_list())
            .unwrap_or(false)
    }
}

/// Convenience methods that delegate to trait implementations
pub struct AstUtils;

impl AstUtils {
    /// Get span text for convenience
    pub fn span_text<'a>(span: Span, context: &'a LintContext) -> &'a str {
        &context.source[span.start..span.end]
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
        match &expr.expr {
            Expr::Call(call) => call.is_call_to_command(&self.command_name, context),
            _ => false,
        }
    }
}

/// Pattern for matching assignment operations
pub struct AssignmentPattern;

impl PatternMatcher for AssignmentPattern {
    fn matches(&self, expr: &Expression, _context: &LintContext) -> bool {
        expr.is_assignment()
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
        expr.refers_to_variable(context, &self.variable_name)
    }
}

/// Utility for extracting loop variables from common control structures
pub struct LoopVariableExtractor;

impl LoopVariableExtractor {
    /// Extract loop variable name from 'each' command
    pub fn from_each_call(call: &Call, context: &LintContext) -> Option<String> {
        let first_arg = call.get_first_positional_arg()?;
        let block_id = first_arg.extract_block_id()?;

        let block = context.working_set.get_block(block_id);
        let var_id = block.signature.required_positional.first()?.var_id?;

        let var = context.working_set.get_variable(var_id);
        Some(AstUtils::span_text(var.declaration_span, context).to_string())
    }

    /// Extract loop variable name from 'for' command
    pub fn from_for_call(call: &Call, context: &LintContext) -> Option<String> {
        let var_arg = call.get_first_positional_arg()?;
        var_arg.extract_variable_name(context)
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
            description: format!("Rename {item_type} '{current_name}' to '{suggested_name}'")
                .into(),
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

        name.chars().enumerate().all(|(i, c)| match c {
            'a'..='z' | '0'..='9' => true,
            '-' => {
                if i == 0 {
                    return false;
                }
                name.chars().nth(i + 1) != Some('-')
            }
            _ => false,
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

    /// Check if a command is a variable declaration, returns (is_declaration,
    /// is_mutable)
    pub fn is_var_declaration(decl_name: &str) -> Option<bool> {
        match decl_name {
            "let" => Some(false),
            "mut" => Some(true),
            _ => None,
        }
    }

    /// Extract the first argument (name) from a declaration call
    pub fn extract_declaration_name(call: &Call, context: &LintContext) -> Option<(String, Span)> {
        let name_arg = call.get_first_positional_arg()?;
        let name = context.source.get(name_arg.span.start..name_arg.span.end)?;
        Some((name.to_string(), name_arg.span))
    }

    /// Extract function definition information from a call
    pub fn extract_function_definition(
        call: &Call,
        context: &LintContext,
    ) -> Option<(BlockId, String)> {
        let decl_name = call.get_call_name(context);
        if !Self::is_def_command(&decl_name) {
            return None;
        }

        // First argument is the function name
        let name_arg = call.get_first_positional_arg()?;
        let name = AstUtils::span_text(name_arg.span, context);

        // Third argument is the function body block (can be Block or Closure)
        let body_expr = call.get_positional_arg(2)?;
        let block_id = body_expr.extract_block_id()?;

        Some((block_id, name.to_string()))
    }

    /// Extract variable declaration info (var_id, name, span) from let/mut
    /// calls
    pub fn extract_variable_declaration(
        call: &Call,
        context: &LintContext,
    ) -> Option<(nu_protocol::VarId, String, Span)> {
        let decl_name = call.get_call_name(context);
        if !matches!(decl_name.as_str(), "let" | "mut") {
            return None;
        }

        let var_arg = call.get_first_positional_arg()?;

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

        if !expr.is_assignment() {
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
    pub fn extract_field_access(
        expr: &Expression,
        field_name: &str,
    ) -> Option<(nu_protocol::VarId, Span)> {
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

    /// Find which function contains a given span (returns the most specific
    /// one)
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
            &|inner_expr| inner_expr.extract_block_id().into_iter().collect(),
            &mut blocks,
        );
        blocks
    }

    /// Collect all function definitions with their names and block IDs
    pub fn collect_function_definitions(
        context: &LintContext,
    ) -> std::collections::HashMap<BlockId, String> {
        use nu_protocol::ast::Traverse;

        let mut functions = Vec::new();
        context.ast.flat_map(
            context.working_set,
            &|expr| {
                let Expr::Call(call) = &expr.expr else {
                    return vec![];
                };
                DeclarationUtils::extract_function_definition(call, context)
                    .into_iter()
                    .collect()
            },
            &mut functions,
        );
        functions.into_iter().collect()
    }
}
