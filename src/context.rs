use std::collections::HashMap;

use nu_protocol::{
    BlockId, DeclId, Span,
    ast::{Block, Expr, Expression, Traverse},
    engine::{Command, EngineState, StateWorkingSet},
};

use crate::{ast::call::CallExt, violation::Violation};

/// Context containing all lint information (source, AST, and engine state)
/// Rules can use whatever they need from this context
pub struct LintContext<'a> {
    pub source: &'a str,
    pub ast: &'a Block,
    pub engine_state: &'a EngineState,
    pub working_set: &'a StateWorkingSet<'a>,
}

impl LintContext<'_> {
    /// Collect all rule violations using a closure over expressions
    pub(crate) fn collect_rule_violations<F>(&self, collector: F) -> Vec<Violation>
    where
        F: Fn(&Expression, &Self) -> Vec<Violation>,
    {
        let mut violations = Vec::new();

        let f = |expr: &Expression| collector(expr, self);

        // Visit main AST
        self.ast.flat_map(self.working_set, &f, &mut violations);

        violations
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
    #[must_use]
    pub fn find_declaration_span(&self, name: &str) -> Span {
        // Use more efficient string search for function declarations
        // Look for function declarations starting with "def " or "export def "

        // Try "def <name>" first (most common case)
        if let Some(pos) = self.source.find(&format!("def {name}")) {
            let name_start = pos + 4; // "def ".len() == 4
            return Span::new(name_start, name_start + name.len());
        }

        // Try "export def <name>"
        if let Some(pos) = self.source.find(&format!("export def {name}")) {
            let name_start = pos + 11; // "export def ".len() == 11
            return Span::new(name_start, name_start + name.len());
        }

        // Fallback to simple name search
        self.source.find(name).map_or_else(
            || self.ast.span.unwrap_or_else(Span::unknown),
            |name_pos| Span::new(name_pos, name_pos + name.len()),
        )
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

    /// Collect all function definitions with their names and block IDs
    #[must_use]
    pub fn collect_function_definitions(&self) -> HashMap<BlockId, String> {
        let mut functions = Vec::new();
        self.ast.flat_map(
            self.working_set,
            &|expr| {
                let Expr::Call(call) = &expr.expr else {
                    return vec![];
                };
                call.extract_function_definition(self).into_iter().collect()
            },
            &mut functions,
        );
        functions.into_iter().collect()
    }

    /// Check if a function is exported
    #[must_use]
    pub fn is_exported_function(&self, function_name: &str) -> bool {
        self.source.contains(&format!("export def {function_name}"))
    }
}

#[cfg(test)]
impl LintContext<'_> {
    /// Helper to create a test context with stdlib commands loaded
    #[track_caller]
    pub fn test_with_parsed_source<F, R>(source: &str, f: F) -> R
    where
        F: for<'b> FnOnce(LintContext<'b>) -> R,
    {
        use nu_parser::parse;

        fn create_engine_with_stdlib() -> EngineState {
            use nu_protocol::engine::StateWorkingSet;

            let engine_state = nu_cmd_lang::create_default_context();
            let engine_state = nu_command::add_shell_command_context(engine_state);
            let mut engine_state = nu_cli::add_cli_context(engine_state);

            // Add print command (it's in nu-cli but not added by add_cli_context)
            let delta = {
                let mut working_set = StateWorkingSet::new(&engine_state);
                working_set.add_decl(Box::new(nu_cli::Print));
                working_set.render()
            };

            if let Err(err) = engine_state.merge_delta(delta) {
                eprintln!("Error adding Print command: {err:?}");
            }

            engine_state
        }

        let engine_state = create_engine_with_stdlib();
        let mut working_set = StateWorkingSet::new(&engine_state);
        let block = parse(&mut working_set, None, source.as_bytes(), false);

        let context = LintContext {
            source,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
        };

        f(context)
    }
}
