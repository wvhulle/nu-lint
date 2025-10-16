use crate::ast_walker::{AstVisitor, VisitContext};
use miette::{Diagnostic, SourceSpan};
use nu_protocol::{
    DeclId, Span,
    ast::{Block, Expression},
    engine::{Command, EngineState, StateWorkingSet},
};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleCategory {
    Style,
    BestPractices,
    Performance,
    Documentation,
    TypeSafety,
}

impl std::fmt::Display for RuleCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleCategory::Style => write!(f, "style"),
            RuleCategory::BestPractices => write!(f, "best-practices"),
            RuleCategory::Performance => write!(f, "performance"),
            RuleCategory::Documentation => write!(f, "documentation"),
            RuleCategory::TypeSafety => write!(f, "type-safety"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Violation {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub suggestion: Option<String>,
    pub fix: Option<Fix>,
    pub file: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Fix {
    pub description: String,
    pub replacements: Vec<Replacement>,
}

#[derive(Debug, Clone)]
pub struct Replacement {
    pub span: Span,
    pub new_text: String,
}

pub struct LintContext<'a> {
    pub source: &'a str,
    pub ast: &'a Block,
    pub engine_state: &'a EngineState,
    pub working_set: &'a StateWorkingSet<'a>,
    pub file_path: Option<&'a Path>,
}

impl LintContext<'_> {
    /// Get the range of declaration IDs that were added during parsing (the delta)
    /// Returns (`base_count`, `total_count`) for iterating: `base_count..total_count`
    #[must_use]
    pub fn new_decl_range(&self) -> (usize, usize) {
        let base_count = self.engine_state.num_decls();
        let total_count = self.working_set.num_decls();
        (base_count, total_count)
    }

    /// Iterator over newly added user-defined function declarations
    /// Filters out built-in functions (those with spaces or starting with '_')
    pub fn new_user_functions(&self) -> impl Iterator<Item = (usize, &dyn Command)> + '_ {
        let (base_count, total_count) = self.new_decl_range();
        (base_count..total_count)
            .map(|decl_id| (decl_id, self.working_set.get_decl(DeclId::new(decl_id))))
            .filter(|(_, decl)| {
                let name = &decl.signature().name;
                !name.contains(' ') && !name.starts_with('_')
            })
    }

    /// Find the span of a function/declaration name in the source code
    /// Returns a span pointing to the first occurrence of the name, or a fallback span
    pub fn find_declaration_span(&self, name: &str) -> Span {
        if let Some(name_pos) = self.source.find(name) {
            Span::new(name_pos, name_pos + name.len())
        } else {
            self.ast.span.unwrap_or_else(Span::unknown)
        }
    }

    /// Find violations by applying a conditional predicate to regex matches
    ///
    /// This is the most flexible regex helper. Use when you need to:
    /// - Filter matches conditionally (not all matches are violations)
    /// - Customize both message and suggestion per match
    /// - Access the full `regex::Match` object for complex logic
    ///
    /// # Arguments
    /// * `pattern` - The regex pattern to match
    /// * `rule_id` - The rule ID for violations
    /// * `severity` - The severity level
    /// * `predicate` - Function that returns Some((message, suggestion)) if violation should be created
    ///
    /// # Example
    /// ```ignore
    /// context.violations_from_regex_if(
    ///     &regex,
    ///     "BP002",
    ///     Severity::Warning,
    ///     |mat| {
    ///         if some_condition(mat.as_str()) {
    ///             Some(("Error message".to_string(), Some("Suggestion".to_string())))
    ///         } else {
    ///             None
    ///         }
    ///     }
    /// )
    /// ```
    pub fn violations_from_regex_if<F>(
        &self,
        pattern: &regex::Regex,
        rule_id: &str,
        severity: Severity,
        predicate: F,
    ) -> Vec<Violation>
    where
        F: Fn(regex::Match) -> Option<(String, Option<String>)>,
    {
        pattern
            .find_iter(self.source)
            .filter_map(|mat| {
                predicate(mat).map(|(message, suggestion)| Violation {
                    rule_id: rule_id.to_string(),
                    severity,
                    message,
                    span: Span::new(mat.start(), mat.end()),
                    suggestion,
                    fix: None,
                    file: None,
                })
            })
            .collect()
    }

    /// Find violations from all regex matches with dynamic messages
    ///
    /// Use when every match is a violation, but the message varies based on matched text.
    ///
    /// # Arguments
    /// * `pattern` - The regex pattern to match
    /// * `rule_id` - The rule ID for violations
    /// * `severity` - The severity level
    /// * `message_fn` - Function to generate message from the matched text
    /// * `suggestion` - Optional suggestion text (same for all matches)
    ///
    /// # Example
    /// ```ignore
    /// context.violations_from_regex_with_message(
    ///     &regex,
    ///     "S001",
    ///     Severity::Warning,
    ///     |text| format!("Found pattern: {}", text),
    ///     Some("Fix suggestion")
    /// )
    /// ```
    pub fn violations_from_regex_with_message<F>(
        &self,
        pattern: &regex::Regex,
        rule_id: &str,
        severity: Severity,
        message_fn: F,
        suggestion: Option<&String>,
    ) -> Vec<Violation>
    where
        F: Fn(&str) -> String,
    {
        self.violations_from_regex_if(pattern, rule_id, severity, |mat| {
            Some((message_fn(mat.as_str()), suggestion.cloned()))
        })
    }

    /// Find violations from all regex matches with a fixed message
    ///
    /// Use when every match is a violation with the same message and suggestion.
    /// This is the simplest regex helper.
    ///
    /// # Arguments
    /// * `pattern` - The regex pattern to match
    /// * `rule_id` - The rule ID for violations
    /// * `severity` - The severity level
    /// * `message` - The message for all violations
    /// * `suggestion` - Optional suggestion text
    ///
    /// # Example
    /// ```ignore
    /// context.violations_from_regex(
    ///     &pattern,
    ///     "S010",
    ///     Severity::Info,
    ///     "Use 'is-not-empty' instead of 'not ... is-empty'",
    ///     Some("Replace with 'is-not-empty'")
    /// )
    /// ```
    pub fn violations_from_regex(
        &self,
        pattern: &regex::Regex,
        rule_id: &str,
        severity: Severity,
        message: impl Into<String>,
        suggestion: Option<String>,
    ) -> Vec<Violation> {
        let message = message.into();
        self.violations_from_regex_if(pattern, rule_id, severity, |_| {
            Some((message.clone(), suggestion.clone()))
        })
    }

    /// Walk the AST using a visitor pattern
    ///
    /// This is the primary method for AST-based rules. The visitor will be called
    /// for each relevant AST node type. This walks both the main AST block and
    /// all blocks accessible through function declarations.
    ///
    /// # Example
    /// ```ignore
    /// let mut visitor = MyCustomVisitor::new();
    /// context.walk_ast(&mut visitor);
    /// let violations = visitor.get_violations();
    /// ```
    pub fn walk_ast<V: AstVisitor>(&self, visitor: &mut V) {
        let visit_context = VisitContext::new(self.working_set, self.source);

        // Visit the main AST block
        visitor.visit_block(self.ast, &visit_context);

        // Visit function bodies by iterating through user-defined functions
        for (_decl_id, decl) in self.new_user_functions() {
            if let Some(block_id) = decl.block_id() {
                let block = self.working_set.get_block(block_id);
                visitor.visit_block(block, &visit_context);
            }
        }
    }

    /// Collect all expressions matching a predicate
    ///
    /// This is a convenient helper for rules that need to find specific types
    /// of expressions in the AST.
    ///
    /// # Example
    /// ```ignore
    /// // Find all binary operations
    /// let binary_ops = context.collect_expressions(|expr| {
    ///     matches!(expr.expr, nu_protocol::ast::Expr::BinaryOp(_, _, _))
    /// });
    /// ```
    pub fn collect_expressions<F>(&self, predicate: F) -> Vec<(Expression, Span)>
    where
        F: Fn(&Expression) -> bool,
    {
        let mut collector = crate::ast_walker::ExpressionCollector::new(predicate);
        self.walk_ast(&mut collector);
        collector.expressions
    }

    /// Collect all variable declarations in the AST
    #[must_use]
    pub fn collect_var_declarations(&self) -> Vec<(nu_protocol::VarId, Span)> {
        let mut collector = crate::ast_walker::VarDeclCollector::new();
        self.walk_ast(&mut collector);
        collector.var_decls
    }

    /// Collect all function calls in the AST
    #[must_use]
    pub fn collect_function_calls(&self) -> Vec<(nu_protocol::ast::Call, Span)> {
        let mut collector = crate::ast_walker::CallCollector::new();
        self.walk_ast(&mut collector);
        collector.calls
    }

    /// Get the text content of a span
    #[must_use]
    pub fn get_span_contents(&self, span: Span) -> &str {
        crate::parser::get_span_contents(self.source, span)
    }

    /// Create a simple Fix with a single replacement
    /// This is a convenience method for creating fixes that replace one span with new text
    pub fn create_simple_fix(
        &self,
        description: impl Into<String>,
        span: Span,
        new_text: impl Into<String>,
    ) -> Fix {
        Fix {
            description: description.into(),
            replacements: vec![Replacement {
                span,
                new_text: new_text.into(),
            }],
        }
    }
}

pub trait Rule: Send + Sync {
    fn id(&self) -> &str;
    fn category(&self) -> RuleCategory;
    fn severity(&self) -> Severity;
    fn description(&self) -> &str;

    fn check(&self, context: &LintContext) -> Vec<Violation>;
}

#[derive(Error, Debug, Diagnostic)]
pub enum LintError {
    #[error("Failed to parse file: {0}")]
    ParseError(String),

    #[error("Failed to read file: {0}")]
    #[diagnostic(code(nu_lint::io_error))]
    IoError(#[from] std::io::Error),

    #[error("Invalid configuration: {0}")]
    #[diagnostic(code(nu_lint::config_error))]
    ConfigError(String),
}

impl Violation {
    #[must_use]
    pub fn to_source_span(&self) -> SourceSpan {
        SourceSpan::from((self.span.start, self.span.end - self.span.start))
    }
}

#[cfg(test)]
impl<'a> LintContext<'a> {
    /// Helper to create a test context with dummy values
    /// Only the source is used by regex-based rules
    pub fn test_from_source(source: &'a str) -> Self {
        use nu_protocol::engine::{EngineState, StateWorkingSet};
        use std::sync::OnceLock;

        // Thread-safe lazy initialization using OnceLock (no unsafe needed)
        static DUMMY_BLOCK: OnceLock<nu_protocol::ast::Block> = OnceLock::new();
        static DUMMY_ENGINE: OnceLock<EngineState> = OnceLock::new();

        // Initialize static values on first access
        let block = DUMMY_BLOCK.get_or_init(nu_protocol::ast::Block::default);
        let engine_state = DUMMY_ENGINE.get_or_init(EngineState::new);

        // Create a working set on the stack (can't be static due to lifetime constraints)
        // This is safe because we're only using it for the lifetime of this test
        let working_set = StateWorkingSet::new(engine_state);

        // SAFETY: We need to extend the lifetime of working_set to 'a for the return type.
        // This is safe in test context because:
        // 1. The engine_state is static and lives for the entire program
        // 2. The working_set is only used within the test and not mutated
        // 3. Tests are single-threaded per test case
        let working_set_ref: &'a StateWorkingSet<'a> = unsafe { std::mem::transmute(&working_set) };

        Self {
            source,
            ast: block,
            engine_state,
            working_set: working_set_ref,
            file_path: None,
        }
    }
}
