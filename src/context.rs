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
    /// Raw source string - DO NOT slice directly with spans!
    /// Use `get_span_text()` or `working_set.get_span_contents()` instead.
    /// Direct slicing will fail when the engine state contains loaded files (like stdlib).
    source: &'a str,
    pub ast: &'a Block,
    pub engine_state: &'a EngineState,
    pub working_set: &'a StateWorkingSet<'a>,
    /// Offset of the current file in the virtual span space
    /// Used to normalize spans to file-relative positions for fixes
    file_offset: usize,
}

impl<'a> LintContext<'a> {
    /// Create a new LintContext
    /// Only the engine module should construct this
    pub(crate) fn new(
        source: &'a str,
        ast: &'a Block,
        engine_state: &'a EngineState,
        working_set: &'a StateWorkingSet<'a>,
        file_offset: usize,
    ) -> Self {
        Self {
            source,
            ast,
            engine_state,
            working_set,
            file_offset,
        }
    }

    /// Normalize a span to be relative to the current file
    /// This is needed because spans are absolute positions in the virtual file space,
    /// but we need file-relative positions for applying fixes
    #[must_use]
    pub fn normalize_span(&self, span: Span) -> Span {
        Span::new(
            span.start.saturating_sub(self.file_offset),
            span.end.saturating_sub(self.file_offset),
        )
    }

    /// Get the raw source text
    ///
    /// # Safety
    ///
    /// DO NOT slice the returned string with span indices! When the engine state
    /// contains loaded files (like stdlib), spans are absolute positions in a virtual
    /// file space and won't match this source string's indices.
    ///
    /// Only use this for operations like:
    /// - Line counting (`source().lines().count()`)
    /// - Pattern searching (`source().contains()`)
    /// - Getting full source length (`source().len()`)
    ///
    /// For span-based text extraction, ALWAYS use `get_span_text()` instead.
    #[must_use]
    pub unsafe fn source(&self) -> &str {
        self.source
    }

    /// Safely get text for a span by using the working set
    /// This properly handles file offsets from loaded stdlib files
    #[must_use]
    pub fn get_span_text(&self, span: Span) -> &str {
        std::str::from_utf8(self.working_set.get_span_contents(span))
            .unwrap_or("")
    }

    /// Get the full source length (safe - doesn't involve span slicing)
    #[must_use]
    pub fn source_len(&self) -> usize {
        self.source.len()
    }

    /// Check if source contains a substring (safe - doesn't involve span slicing)
    #[must_use]
    pub fn source_contains(&self, pattern: &str) -> bool {
        self.source.contains(pattern)
    }

    /// Get source lines iterator (safe - doesn't involve span slicing)
    pub fn source_lines(&self) -> impl Iterator<Item = &str> {
        self.source.lines()
    }

    /// Get source text before a position (safe - doesn't involve span slicing)
    /// Use this for checking what comes before a span (e.g., for doc comments)
    #[must_use]
    pub fn source_before(&self, position: usize) -> &str {
        self.source.get(..position).unwrap_or("")
    }

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
    #[must_use]
    pub fn find_declaration_span(&self, name: &str) -> Span {
        const PATTERNS: &[(&str, &str, usize)] = &[
            ("def ", "", 4),
            ("def \"", "\"", 5),
            ("export def ", "", 11),
            ("export def \"", "\"", 12),
        ];

        for (prefix, suffix, offset) in PATTERNS {
            let pattern = format!("{prefix}{name}{suffix}");
            if let Some(pos) = self.source.find(&pattern) {
                let start = pos + offset;
                return Span::new(start, start + name.len());
            }
        }

        self.source.find(name).map_or_else(
            || self.ast.span.unwrap_or_else(Span::unknown),
            |pos| Span::new(pos, pos + name.len()),
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

        let mut working_set = StateWorkingSet::new(&engine_state);
        let block = parse(&mut working_set, None, source.as_bytes(), false);

        let context = LintContext::new(source, &block, &engine_state, &working_set, 0);

        f(context)
    }
}
