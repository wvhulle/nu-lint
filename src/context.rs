use std::path::Path;

use nu_protocol::{
    DeclId, Span,
    ast::{Block, Expression, FindMapResult, Traverse},
    engine::{Command, EngineState, StateWorkingSet},
};

use crate::lint::{RuleViolation, Violation};

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
    pub fn violations_from_regex<MatchPredicate>(
        &self,
        pattern: &regex::Regex,
        rule_id: &'static str,
        predicate: MatchPredicate,
    ) -> Vec<RuleViolation>
    where
        MatchPredicate: Fn(regex::Match) -> Option<(String, Option<String>)>,
    {
        pattern
            .find_iter(self.source)
            .filter_map(|mat| {
                predicate(mat).map(|(message, suggestion)| {
                    let violation = RuleViolation::new_dynamic(
                        rule_id,
                        message,
                        Span::new(mat.start(), mat.end()),
                    );
                    match suggestion {
                        Some(sug) => violation.with_suggestion_dynamic(sug),
                        None => violation,
                    }
                })
            })
            .collect()
    }

    /// Collect all violations using a closure over expressions (Traverse-based)
    ///
    /// This method uses Nushell's upstream `Traverse` trait to walk the AST
    /// and collect violations. The collector function is called for each
    /// expression in the AST and should return a vector of violations.
    pub fn collect_violations<F>(&self, collector: F) -> Vec<Violation>
    where
        F: Fn(&Expression, &Self) -> Vec<Violation>,
    {
        let mut violations = Vec::new();

        let f = |expr: &Expression| collector(expr, self);

        // Visit main AST
        self.ast.flat_map(self.working_set, &f, &mut violations);

        violations
    }

    /// Collect all rule violations using a closure over expressions
    /// (Traverse-based)
    ///
    /// This method uses Nushell's upstream `Traverse` trait to walk the AST
    /// and collect rule violations. The collector function is called for each
    /// expression in the AST and should return a vector of rule violations.
    pub fn collect_rule_violations<F>(&self, collector: F) -> Vec<RuleViolation>
    where
        F: Fn(&Expression, &Self) -> Vec<RuleViolation>,
    {
        let mut violations = Vec::new();

        let f = |expr: &Expression| collector(expr, self);

        // Visit main AST
        self.ast.flat_map(self.working_set, &f, &mut violations);

        violations
    }

    /// Find first match using `find_map` (Traverse-based)
    ///
    /// This method uses Nushell's upstream `Traverse` trait to search the AST
    /// for the first matching expression. The finder function should return
    /// `FindMapResult::Found(value)` to return a value, `FindMapResult::Stop`
    /// to stop searching, or `FindMapResult::Continue` to continue searching.
    pub fn find_match<T, F>(&self, finder: F) -> Option<T>
    where
        F: Fn(&Expression) -> FindMapResult<T>,
    {
        self.ast.find_map(self.working_set, &finder)
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
        use nu_protocol::engine::StateWorkingSet;

        fn create_engine_with_stdlib() -> nu_protocol::engine::EngineState {
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
            file_path: None,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
        };

        f(context)
    }
}
