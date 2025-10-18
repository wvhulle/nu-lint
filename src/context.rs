use std::path::Path;

use nu_protocol::{
    DeclId, Span,
    ast::Block,
    engine::{Command, EngineState, StateWorkingSet},
};

use crate::{
    lint::{Severity, Violation},
    visitor::{AstVisitor, VisitContext},
};

/// Context containing all lint information (source, AST, and engine state)
/// Rules can use whatever they need from this context
pub struct LintContext<'a> {
    pub source: &'a str,
    pub file_path: Option<&'a Path>,
    pub ast: &'a Block,
    pub engine_state: &'a EngineState,
    pub working_set: &'a StateWorkingSet<'a>,
}

impl LintContext<'_> {
    /// Find violations by applying a conditional predicate to regex matches
    ///
    /// This is a helper for regex-based rules. Use when you need to:
    /// - Filter matches conditionally (not all matches are violations)
    /// - Customize both message and suggestion per match
    /// - Access the full `regex::Match` object for complex logic
    ///
    /// # Arguments
    /// * `pattern` - The regex pattern to match
    /// * `rule_id` - The rule ID for violations
    /// * `severity` - The severity level
    /// * `predicate` - Function that returns Some((message, suggestion)) if
    ///   violation should be created
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
                    rule_id: rule_id.to_string().into(),
                    severity,
                    message: message.into(),
                    span: Span::new(mat.start(), mat.end()),
                    suggestion: suggestion.map(Into::into),
                    fix: None,
                    file: None,
                })
            })
            .collect()
    }

    /// Walk the AST using a visitor pattern
    ///
    /// This is the primary method for AST-based rules. The visitor will be
    /// called for each relevant AST node type. This walks both the main AST
    /// block and all blocks accessible through function declarations.
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
    /// Returns a span pointing to the first occurrence of the name, or a
    /// fallback span
    pub fn find_declaration_span(&self, name: &str) -> Span {
        // Use more efficient string search for function names
        // Look for function declarations starting with "def " or "export def "
        let patterns = [
            format!("def {name}"),
            format!("export def {name}"),
        ];

        for pattern in &patterns {
            if let Some(pos) = self.source.find(pattern) {
                // Find the start of the function name within the pattern
                let name_start = pos + pattern.len() - name.len();
                return Span::new(name_start, name_start + name.len());
            }
        }

        // Fallback to simple name search
        if let Some(name_pos) = self.source.find(name) {
            Span::new(name_pos, name_pos + name.len())
        } else {
            self.ast.span.unwrap_or_else(Span::unknown)
        }
    }

    /// Get the range of declaration IDs that were added during parsing (the
    /// delta) Returns (`base_count`, `total_count`) for iterating:
    /// `base_count..total_count`
    #[must_use]
    pub fn new_decl_range(&self) -> (usize, usize) {
        let base_count = self.engine_state.num_decls();
        let total_count = self.working_set.num_decls();
        (base_count, total_count)
    }
}

#[cfg(test)]
impl LintContext<'_> {
    /// Helper to create a test context with stdlib commands loaded
    pub fn test_with_parsed_source<F, R>(source: &str, f: F) -> R
    where
        F: for<'b> FnOnce(LintContext<'b>) -> R,
    {
        use nu_parser::parse;
        use nu_protocol::engine::StateWorkingSet;

        fn create_engine_with_stdlib() -> nu_protocol::engine::EngineState {
            let engine_state = nu_cmd_lang::create_default_context();
            nu_command::add_shell_command_context(engine_state)
        }

        let engine_state = create_engine_with_stdlib();
        let mut working_set = StateWorkingSet::new(&engine_state);
        let block = parse(&mut working_set, None, source.as_bytes(), false);

        let context = LintContext {
            source,
            file_path: None,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
        };

        f(context)
    }
}