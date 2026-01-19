use std::{collections::BTreeSet, ops::ControlFlow, str::from_utf8, vec::Vec};

use nu_protocol::{
    Span,
    ast::{Block, Expr, Expression, Traverse},
    engine::{EngineState, StateWorkingSet},
};

#[cfg(test)]
use crate::violation;
use crate::{
    Config,
    ast::{self, call::CallExt, string::StringFormat},
    span::FileSpan,
    violation::Detection,
};

/// Fix data for external command alternatives
pub struct ExternalCmdFixData<'a> {
    /// Argument expressions from the external call
    pub args: Box<[&'a Expression]>,
    pub expr_span: Span,
}

impl ExternalCmdFixData<'_> {
    /// Get argument text content for each argument.
    ///
    /// For string literals, returns the unquoted content.
    /// For other expressions (variables, subexpressions), returns the source
    /// text.
    ///
    /// This is the primary API for parsing command arguments.
    pub fn arg_texts<'b>(&'b self, context: &'b LintContext<'b>) -> impl Iterator<Item = &'b str> {
        self.args.iter().map(move |expr| match &expr.expr {
            Expr::String(s) | Expr::RawString(s) => s.as_str(),
            _ => context.expr_text(expr),
        })
    }

    /// Get string format information for arguments that need quote
    /// preservation.
    ///
    /// Returns `Some(StringFormat)` for string literals (with quote type info).
    /// Returns `None` for non-string expressions (variables, subexpressions,
    /// etc.).
    ///
    /// Use this when generating replacement text that must preserve quote
    /// styles.
    pub fn arg_formats(&self, context: &LintContext) -> Vec<Option<StringFormat>> {
        self.args
            .iter()
            .map(|expr| StringFormat::from_expression(expr, context))
            .collect()
    }

    /// Check if an argument is a string literal (safe to extract unquoted
    /// content).
    pub fn arg_is_string(&self, index: usize) -> bool {
        self.args.get(index).is_some_and(|expr| {
            matches!(
                &expr.expr,
                Expr::String(_) | Expr::RawString(_) | Expr::StringInterpolation(_)
            )
        })
    }
}

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

    /// Check if a global span is within the user's file bounds
    #[must_use]
    pub const fn span_in_user_file(&self, span: Span) -> bool {
        let file_end = self.file_offset + self.source.len();
        span.start >= self.file_offset && span.end <= file_end
    }

    /// Get the source length of the user's file
    #[must_use]
    pub const fn source_len(&self) -> usize {
        self.source.len()
    }

    /// Get text for an AST span
    #[must_use]
    pub fn span_text(&self, span: Span) -> &str {
        from_utf8(self.working_set.get_span_contents(span))
            .expect("span contents should be valid UTF-8")
    }

    #[must_use]
    pub fn expr_text(&self, expr: &Expression) -> &str {
        self.span_text(expr.span)
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

    /// Collect spans of all calls to the specified commands
    #[must_use]
    pub fn collect_command_spans(&self, commands: &[&str]) -> Vec<Span> {
        let mut spans = Vec::new();
        self.ast.flat_map(
            self.working_set,
            &|expr| {
                if let Expr::Call(call) = &expr.expr {
                    let cmd_name = call.get_call_name(self);
                    if commands.iter().any(|&cmd| cmd == cmd_name) {
                        return vec![expr.span];
                    }
                }
                vec![]
            },
            &mut spans,
        );
        spans
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

    pub(crate) fn detect_single<F>(&self, detector: F) -> Vec<Detection>
    where
        F: Fn(&Expression, &Self) -> Option<Detection>,
    {
        let mut violations = Vec::new();
        let f = |expr: &Expression| {
            detector(expr, self).map_or_else(Vec::new, |detection| vec![detection])
        };
        self.ast.flat_map(self.working_set, &f, &mut violations);
        violations
    }

    /// Traverse the AST with parent context, calling the callback for each
    /// expression with its parent expression (if any).
    ///
    /// This builds on top of the `Traverse` trait but adds parent tracking,
    /// which is useful for rules that need to know the context of an
    /// expression (e.g., whether a string is in command position).
    ///
    /// The callback returns `ControlFlow::Continue(())` to recurse into
    /// children, or `ControlFlow::Break(())` to skip this expression's
    /// children.
    pub(crate) fn traverse_with_parent<F>(&self, mut callback: F)
    where
        F: FnMut(&Expression, Option<&Expression>) -> ControlFlow<()>,
    {
        use crate::ast::block::BlockExt;

        self.ast.traverse_with_parent(self, None, &mut callback);
    }

    /// Range of declaration IDs added during parsing: `base..total`
    #[must_use]
    pub fn new_decl_range(&self) -> (usize, usize) {
        let base_count = self.engine_state.num_decls();
        let total_count = self.working_set.num_decls();
        (base_count, total_count)
    }

    /// Collect all function definitions
    #[must_use]
    pub fn custom_commands(&self) -> BTreeSet<ast::declaration::CustomCommandDef> {
        let mut functions = Vec::new();
        self.ast.flat_map(
            self.working_set,
            &|expr| {
                let Expr::Call(call) = &expr.expr else {
                    return vec![];
                };
                call.custom_command_def(self).into_iter().collect()
            },
            &mut functions,
        );
        functions.into_iter().collect()
    }

    /// Detect external command invocations with custom validation.
    /// This allows rules to check if the arguments can be reliably translated
    /// before reporting a violation.
    ///
    /// The validator function receives the command name, fix data, and context,
    /// and should return `Some(note)` if the invocation should be reported,
    /// or `None` if it should be ignored.
    #[must_use]
    pub fn detect_external_with_validation<'context, F>(
        &'context self,
        external_cmd: &'static str,
        validator: F,
    ) -> Vec<(Detection, ExternalCmdFixData<'context>)>
    where
        F: Fn(&str, &ExternalCmdFixData<'context>, &'context Self) -> Option<&'static str>,
    {
        use nu_protocol::ast::{Expr, ExternalArgument, Traverse};

        let mut results = Vec::new();

        self.ast.flat_map(
            self.working_set,
            &|expr| {
                let Expr::ExternalCall(head, args) = &expr.expr else {
                    return vec![];
                };

                let cmd_text = self.span_text(head.span);
                if cmd_text != external_cmd {
                    return vec![];
                }

                let arg_exprs: Vec<&Expression> = args
                    .iter()
                    .map(|arg| match arg {
                        ExternalArgument::Regular(expr) | ExternalArgument::Spread(expr) => expr,
                    })
                    .collect();

                let fix_data = ExternalCmdFixData {
                    args: arg_exprs.into_boxed_slice(),
                    expr_span: expr.span,
                };

                // Validate if this invocation should be reported
                let Some(note) = validator(cmd_text, &fix_data, self) else {
                    return vec![];
                };

                let detected = Detection::from_global_span(note, expr.span)
                    .with_primary_label(format!("external '{cmd_text}'"));

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
        let (block, working_set, file_offset) = parse_source(engine_state, source.as_bytes());
        let config = Config::default();

        let context = LintContext::new(
            source,
            &block,
            engine_state,
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
