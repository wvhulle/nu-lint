use std::{collections::HashMap, str::from_utf8};

use nu_protocol::{
    BlockId, DeclId, Span,
    ast::{Block, Expr, Expression, Traverse},
    engine::{Command, EngineState, StateWorkingSet},
};

#[cfg(test)]
use crate::violation;
use crate::{
    Config,
    ast::call::CallExt,
    external_commands::{self, ExternalCmdFixData},
    span::FileSpan,
    violation::Detection,
};

/// Context containing all lint information (source, AST, and engine state)
///
/// # Span Handling
///
/// AST spans are in a "global" coordinate system that includes all loaded files
/// (stdlib, etc.). This context encapsulates span translation - rule authors
/// should use the provided methods and never manually slice source with spans.
pub struct LintContext<'a> {
    /// Raw source string of the file being linted (file-relative coordinates)
    source: &'a str,
    pub ast: &'a Block,
    pub engine_state: &'a EngineState,
    pub working_set: &'a StateWorkingSet<'a>,
    /// Byte offset where this file starts in the global span space
    file_offset: usize,
    pub config: &'a Config,
}

impl<'a> LintContext<'a> {
    /// Create a new `LintContext`
    pub(crate) const fn new(
        source: &'a str,
        ast: &'a Block,
        engine_state: &'a EngineState,
        working_set: &'a StateWorkingSet<'a>,
        file_offset: usize,
        config: &'a Config,
    ) -> Self {
        Self {
            source,
            ast,
            engine_state,
            working_set,
            file_offset,
            config,
        }
    }

    #[must_use]
    pub const unsafe fn source(&self) -> &str {
        self.source
    }

    /// Get text for an AST span
    #[must_use]
    pub fn get_span_text(&self, span: Span) -> &str {
        from_utf8(self.working_set.get_span_contents(span))
            .expect("span contents should be valid UTF-8")
    }

    /// Get source text before an AST span
    #[must_use]
    pub fn source_before_span(&self, span: Span) -> &str {
        let file_pos = span.start.saturating_sub(self.file_offset);
        self.source
            .get(..file_pos)
            .expect("file position should be within source bounds")
    }

    /// Get source text after an AST span
    #[must_use]
    pub fn source_after_span(&self, span: Span) -> &str {
        let file_pos = span.end.saturating_sub(self.file_offset);
        self.source
            .get(file_pos..)
            .expect("file position should be within source bounds")
    }

    /// Get source text between two span endpoints (from end of first to start
    /// of second) Returns empty string if the range is invalid
    #[must_use]
    pub fn source_between_span_ends(&self, end_span: Span, start_span: Span) -> &str {
        let file_start = end_span.end.saturating_sub(self.file_offset);
        let file_end = start_span.start.saturating_sub(self.file_offset);

        if file_start >= file_end || file_end > self.source.len() {
            return "";
        }

        &self.source[file_start..file_end]
    }

    /// Count newlines up to a file-relative offset
    #[must_use]
    pub fn count_newlines_before(&self, offset: usize) -> usize {
        let safe_offset = offset.min(self.source.len());
        self.source[..safe_offset]
            .bytes()
            .filter(|&b| b == b'\n')
            .count()
    }

    /// Convert an AST span to file-relative positions for `Replacement` spans
    #[must_use]
    pub const fn normalize_span(&self, span: Span) -> FileSpan {
        FileSpan::new(
            span.start.saturating_sub(self.file_offset),
            span.end.saturating_sub(self.file_offset),
        )
    }

    #[must_use]
    pub fn source_contains(&self, pattern: &str) -> bool {
        self.source.contains(pattern)
    }

    /// Get the format name for a file extension based on available `from`
    /// commands.
    ///
    /// This dynamically queries the engine state for `from <format>` commands
    /// and maps file extensions to their corresponding format names.
    ///
    /// Returns `None` if the extension doesn't have a corresponding `from`
    /// command.
    #[must_use]
    pub fn format_for_extension(&self, filename: &str) -> Option<String> {
        let lower = filename.to_lowercase();

        // Extract extension from filename
        let ext = lower.rsplit('.').next()?;

        // Handle .yml -> yaml alias
        let format = if ext == "yml" { "yaml" } else { ext };

        // Check if `from <format>` command exists
        let from_cmd_name = format!("from {format}");
        self.working_set
            .find_decl(from_cmd_name.as_bytes())
            .is_some()
            .then(|| format.to_string())
    }

    /// Byte offset where this file starts in the global span space
    #[must_use]
    pub const fn file_offset(&self) -> usize {
        self.file_offset
    }

    /// Expand a span to include the full line(s) it occupies
    /// Takes a global AST span and returns a global span
    #[must_use]
    pub fn expand_span_to_full_lines(&self, span: Span) -> Span {
        let bytes = self.source.as_bytes();

        let file_start = span.start.saturating_sub(self.file_offset);
        let file_end = span.end.saturating_sub(self.file_offset);

        let start = bytes[..file_start]
            .iter()
            .rposition(|&b| b == b'\n')
            .map_or(0, |pos| pos + 1);

        let end = bytes[file_end..]
            .iter()
            .position(|&b| b == b'\n')
            .map_or(self.source.len(), |pos| file_end + pos + 1);

        Span::new(start + self.file_offset, end + self.file_offset)
    }

    /// Collect detected violations with associated fix data using a closure
    /// over expressions
    pub(crate) fn detect_with_fix_data<F, D>(&self, collector: F) -> Vec<(Detection, D)>
    where
        F: Fn(&Expression, &Self) -> Vec<(Detection, D)>,
        D: 'a,
    {
        let mut results = Vec::new();
        let f = |expr: &Expression| collector(expr, self);
        self.ast.flat_map(self.working_set, &f, &mut results);
        results
    }

    /// Collect detected violations without fix data (convenience for rules with
    /// `FixData = ()`)
    pub(crate) fn detect<F>(&self, fix_data_collector: F) -> Vec<Detection>
    where
        F: Fn(&Expression, &Self) -> Vec<Detection>,
    {
        let mut violations = Vec::new();
        let f = |expr: &Expression| fix_data_collector(expr, self);
        self.ast.flat_map(self.working_set, &f, &mut violations);
        violations
    }

    /// Iterator over newly added user-defined function declarations
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
    /// Returns a file-relative span since it searches the source string
    #[must_use]
    pub fn find_declaration_span(&self, name: &str) -> FileSpan {
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
                return FileSpan::new(start, start + name.len());
            }
        }

        self.source.find(name).map_or_else(
            || self.normalize_span(self.ast.span.unwrap_or_else(Span::unknown)),
            |pos| FileSpan::new(pos, pos + name.len()),
        )
    }

    /// Range of declaration IDs added during parsing: `base..total`
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
                call.custom_command_def(self)
                    .map(|def| (def.body, def.name))
                    .into_iter()
                    .collect()
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

    /// Detect a specific external command and suggest a builtin alternative.
    /// Returns detected violations with fix data that can be used to generate
    /// fixes.
    #[must_use]
    pub fn external_invocations<'context>(
        &'context self,
        external_cmd: &'static str,
        note: &'static str,
    ) -> Vec<(Detection, ExternalCmdFixData<'context>)> {
        use nu_protocol::ast::{Expr, ExternalArgument, Traverse};

        let mut results = Vec::new();

        self.ast.flat_map(
            self.working_set,
            &|expr| {
                let Expr::ExternalCall(head, args) = &expr.expr else {
                    return vec![];
                };

                let cmd_text = self.get_span_text(head.span);
                if cmd_text != external_cmd {
                    return vec![];
                }

                let detected = Detection::from_global_span(note, expr.span)
                    .with_primary_label(format!("external '{cmd_text}'"));

                let arg_strings: Vec<&str> = args
                    .iter()
                    .map(|arg| match arg {
                        ExternalArgument::Regular(expr) | ExternalArgument::Spread(expr) => {
                            self.get_span_text(expr.span)
                        }
                    })
                    .collect();

                let fix_data = external_commands::ExternalCmdFixData {
                    arg_strings,
                    expr_span: expr.span,
                };

                vec![(detected, fix_data)]
            },
            &mut results,
        );

        results
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
        use crate::engine::{LintEngine, parse_source};

        let engine_state = LintEngine::new_state();
        let (block, working_set, file_offset) = parse_source(&engine_state, source.as_bytes());
        let config = Config::default();

        let context = LintContext::new(
            source,
            &block,
            &engine_state,
            &working_set,
            file_offset,
            &config,
        );

        f(context)
    }

    /// Helper to get normalized violations from source code (matches production
    /// behavior)
    #[track_caller]
    pub fn test_get_violations<F>(source: &str, f: F) -> Vec<violation::Violation>
    where
        F: for<'b> FnOnce(&LintContext<'b>) -> Vec<violation::Violation>,
    {
        Self::test_with_parsed_source(source, |context| {
            let file_offset = context.file_offset();
            let mut violations = f(&context);
            for v in &mut violations {
                v.normalize_spans(file_offset);
            }
            violations
        })
    }
}
