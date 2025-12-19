use std::{borrow::Cow, error::Error, fmt, path::Path};

use miette::{Diagnostic, LabeledSpan, Severity};
use nu_protocol::Span;

use crate::{
    config::LintLevel,
    span::{FileSpan, LintSpan},
};

/// Represents the source of a lint violation (either stdin or a file path)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SourceFile {
    Stdin,
    File(String),
}

impl SourceFile {
    /// Get the file path as a string slice (for display and file operations)
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Stdin => "<stdin>",
            Self::File(path) => path.as_str(),
        }
    }

    /// Convert to Path for file operations
    #[must_use]
    pub fn as_path(&self) -> Option<&Path> {
        match self {
            Self::Stdin => None,
            Self::File(path) => Some(Path::new(path)),
        }
    }

    /// Check if this is stdin
    #[must_use]
    pub const fn is_stdin(&self) -> bool {
        matches!(self, Self::Stdin)
    }
}

impl fmt::Display for SourceFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for SourceFile {
    fn from(s: &str) -> Self {
        Self::File(s.to_string())
    }
}

impl From<String> for SourceFile {
    fn from(s: String) -> Self {
        Self::File(s)
    }
}

impl From<&Path> for SourceFile {
    fn from(p: &Path) -> Self {
        Self::File(p.to_string_lossy().to_string())
    }
}

/// Convert `LintLevel` to miette's `Severity`
impl From<LintLevel> for Severity {
    fn from(level: LintLevel) -> Self {
        match level {
            LintLevel::Deny => Self::Error,
            LintLevel::Warn => Self::Warning,
            LintLevel::Allow => Self::Advice,
        }
    }
}

/// A lint violation with its diagnostic information

#[derive(Debug, Clone)]
pub struct Violation {
    pub rule_id: Option<Cow<'static, str>>,
    pub lint_level: LintLevel,

    /// Short message shown in the warning header
    pub message: Cow<'static, str>,

    /// Primary span in source code where the violation occurs
    pub span: LintSpan,

    /// Optional label text displayed on the primary span underline
    pub primary_label: Option<Cow<'static, str>>,

    /// Additional labeled spans for context
    pub extra_labels: Vec<(LintSpan, Option<String>)>,

    /// Optional detailed explanation shown in the "help:" section
    pub help: Option<Cow<'static, str>>,

    /// Additional informational notes shown after help
    pub notes: Vec<Cow<'static, str>>,

    /// Optional automated fix that can be applied
    pub fix: Option<Fix>,

    pub(crate) file: Option<SourceFile>,

    /// Optional source code content
    pub(crate) source: Option<Cow<'static, str>>,

    /// Optional URL to official Nushell documentation
    pub doc_url: Option<&'static str>,
}

impl Violation {
    /// Create a new violation with an AST span (global coordinates)
    ///
    /// The span will be normalized to file-relative coordinates by the engine.
    #[must_use]
    pub fn new(message: impl Into<Cow<'static, str>>, span: Span) -> Self {
        Self {
            rule_id: None,
            lint_level: LintLevel::Allow,
            message: message.into(),
            span: LintSpan::from(span),
            primary_label: None,
            extra_labels: Vec::new(),
            help: None,
            notes: Vec::new(),
            fix: None,
            file: None,
            source: None,
            doc_url: None,
        }
    }

    /// Create a new violation with a file-relative span
    ///
    /// Use this when the span was computed from the source string directly
    /// (e.g., regex matches, manual line counting).
    #[must_use]
    pub fn with_file_span(message: impl Into<Cow<'static, str>>, span: FileSpan) -> Self {
        Self {
            rule_id: None,
            lint_level: LintLevel::Allow,
            message: message.into(),
            span: LintSpan::File(span),
            primary_label: None,
            extra_labels: Vec::new(),
            help: None,
            notes: Vec::new(),
            fix: None,
            file: None,
            source: None,
            doc_url: None,
        }
    }

    /// Set the rule ID for this violation (used by the engine)
    pub(crate) fn set_rule_id(&mut self, rule_id: &'static str) {
        self.rule_id = Some(Cow::Borrowed(rule_id));
    }

    /// Add detailed help text explaining why this change should be made
    #[must_use]
    pub fn with_help(mut self, help: impl Into<Cow<'static, str>>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Add an automated fix to this violation
    #[must_use]
    pub fn with_fix(mut self, fix: Fix) -> Self {
        self.fix = Some(fix);
        self
    }

    /// Set the lint level for this violation (used by the engine)
    pub(crate) const fn set_lint_level(&mut self, level: LintLevel) {
        self.lint_level = level;
    }

    /// Set the documentation URL for this violation (used by the engine)
    pub(crate) const fn set_doc_url(&mut self, url: Option<&'static str>) {
        self.doc_url = url;
    }

    /// Set the label text displayed on the primary span
    #[must_use]
    pub fn with_primary_label(mut self, label: impl Into<Cow<'static, str>>) -> Self {
        self.primary_label = Some(label.into());
        self
    }

    /// Add a secondary label for context with an AST span
    #[must_use]
    pub fn with_extra_label(mut self, label: impl Into<Cow<'static, str>>, span: Span) -> Self {
        self.extra_labels
            .push((LintSpan::from(span), Some(label.into().to_string())));
        self
    }

    /// Add an unlabeled secondary span for context
    #[must_use]
    pub fn with_extra_span(mut self, span: Span) -> Self {
        self.extra_labels.push((LintSpan::from(span), None));
        self
    }

    /// Notes appear after help text and provide supplementary context.
    #[must_use]
    pub fn with_notes<I, S>(mut self, notes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Cow<'static, str>>,
    {
        self.notes = notes.into_iter().map(Into::into).collect();
        self
    }

    /// Add a single note to this violation
    #[must_use]
    pub fn with_note(mut self, note: impl Into<Cow<'static, str>>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Normalize all spans to be file-relative (called by engine before output)
    pub fn normalize_spans(&mut self, file_offset: usize) {
        // Convert main span to file-relative
        let file_span = self.span.to_file_span(file_offset);
        self.span = LintSpan::File(file_span);

        // Normalize fix replacements
        if let Some(fix) = &mut self.fix {
            for replacement in &mut fix.replacements {
                let file_span = replacement.span.to_file_span(file_offset);
                replacement.span = LintSpan::File(file_span);
            }
        }

        // Normalize extra labels
        self.extra_labels = self
            .extra_labels
            .iter()
            .map(|(span, label)| {
                let file_span = span.to_file_span(file_offset);
                (LintSpan::File(file_span), label.clone())
            })
            .collect();
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for Violation {}

impl Diagnostic for Violation {
    fn code<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        Some(Box::new(format!(
            "{}({})",
            self.lint_level,
            self.rule_id.as_deref().unwrap_or("unknown")
        )))
    }

    fn severity(&self) -> Option<Severity> {
        Some(self.lint_level.into())
    }

    fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        self.help
            .as_ref()
            .map(|h| Box::new(h.clone()) as Box<dyn fmt::Display>)
    }

    fn url<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        self.doc_url
            .map(|url| Box::new(url) as Box<dyn fmt::Display>)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        let span_range = self.span.start()..self.span.end();
        let primary = self.primary_label.as_ref().map_or_else(
            || LabeledSpan::underline(span_range.clone()),
            |label| LabeledSpan::new_primary_with_span(Some(label.to_string()), span_range.clone()),
        );
        let extras = self.extra_labels.iter().map(|(span, label)| {
            LabeledSpan::new_with_span(label.clone(), span.start()..span.end())
        });
        Some(Box::new([primary].into_iter().chain(extras)))
    }
}

/// An automated fix that can be applied to resolve a violation
#[derive(Debug, Clone)]
pub struct Fix {
    /// User-facing explanation of what this fix does
    /// Shown in the "ℹ Available fix:" line (can be multi-line)
    pub explanation: Cow<'static, str>,

    /// The actual code replacements to apply to the file
    pub replacements: Vec<Replacement>,
}

impl Fix {
    /// Create a fix with an explanation and code replacements
    #[must_use]
    pub fn with_explanation(
        explanation: impl Into<Cow<'static, str>>,
        replacements: Vec<Replacement>,
    ) -> Self {
        Self {
            explanation: explanation.into(),
            replacements,
        }
    }
}

/// A single code replacement to apply when fixing a violation
///
/// # Important
///
/// The `replacement_text` field contains the ACTUAL CODE that will be written
/// to the file at the specified span. This is not shown directly to the user
/// (except in the before/after diff), but is what gets applied when the fix
/// runs.
#[derive(Debug, Clone)]
pub struct Replacement {
    /// Span in source code to replace (tracks global vs file-relative)
    pub span: LintSpan,

    /// New text to insert at this location
    pub replacement_text: Cow<'static, str>,
}

impl Replacement {
    /// Create a new code replacement with an AST span (global coordinates)
    #[must_use]
    pub fn new(span: Span, replacement_text: impl Into<Cow<'static, str>>) -> Self {
        Self {
            span: LintSpan::from(span),
            replacement_text: replacement_text.into(),
        }
    }

    /// Create a new code replacement with a file-relative span
    #[must_use]
    pub fn with_file_span(span: FileSpan, replacement_text: impl Into<Cow<'static, str>>) -> Self {
        Self {
            span: LintSpan::File(span),
            replacement_text: replacement_text.into(),
        }
    }

    /// Get the span as file-relative (for output). Panics if not normalized.
    #[must_use]
    pub fn file_span(&self) -> FileSpan {
        match self.span {
            LintSpan::File(f) => f,
            LintSpan::Global(_) => panic!("Span not normalized - call normalize_spans first"),
        }
    }
}
