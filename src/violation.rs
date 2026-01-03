use std::{borrow::Cow, error::Error, fmt, iter::once, path::Path, string::ToString};

use miette::{Diagnostic, LabeledSpan, Severity};
use nu_protocol::Span;

use crate::{
    config::LintLevel,
    span::{FileSpan, LintSpan},
};

/// Represents the source file of a lint violation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SourceFile {
    Stdin,
    File(String),
}

impl SourceFile {
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Stdin => "<stdin>",
            Self::File(path) => path.as_str(),
        }
    }

    #[must_use]
    pub fn as_path(&self) -> Option<&Path> {
        match self {
            Self::Stdin => None,
            Self::File(path) => Some(Path::new(path)),
        }
    }

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

impl From<LintLevel> for Severity {
    fn from(level: LintLevel) -> Self {
        match level {
            LintLevel::Error => Self::Error,
            LintLevel::Warning => Self::Warning,
            LintLevel::Hint => Self::Advice,
        }
    }
}

/// A detection in an external file (stdlib, imported module, etc.)
///
/// This represents a violation that occurs in a file other than the one being
/// linted. It carries its own file path, source content, and file-relative span
/// so it can be rendered with proper context.
#[derive(Debug, Clone)]
pub struct ExternalDetection {
    /// The path to the external file
    pub file: String,
    /// The source content of the external file
    pub source: String,
    /// File-relative span within the external file
    pub span: FileSpan,
    /// The error message for this external location
    pub message: String,
    /// Optional label for the span
    pub label: Option<String>,
}

impl ExternalDetection {
    #[must_use]
    pub fn new(
        file: impl Into<String>,
        source: impl Into<String>,
        span: FileSpan,
        message: impl Into<String>,
    ) -> Self {
        Self {
            file: file.into(),
            source: source.into(),
            span,
            message: message.into(),
            label: None,
        }
    }

    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// A detected violation from a lint rule (before fix is attached).
///
/// This type is returned by `LintRule::detect()` and deliberately has no `fix`
/// field. The engine is responsible for calling `LintRule::fix()` separately
/// and combining the results into a full `Violation`.
#[derive(Debug, Clone)]
pub struct Detection {
    pub message: Cow<'static, str>,
    pub span: LintSpan,
    pub primary_label: Option<Cow<'static, str>>,
    pub extra_labels: Vec<(LintSpan, Option<String>)>,
    /// Related detections in external files (stdlib, imported modules, etc.)
    pub external_detections: Vec<ExternalDetection>,
}

impl Detection {
    /// Create a new violation with an AST span (global coordinates)
    #[must_use]
    pub fn from_global_span(message: impl Into<Cow<'static, str>>, global_span: Span) -> Self {
        Self {
            message: message.into(),
            span: LintSpan::from(global_span),
            primary_label: None,
            extra_labels: Vec::new(),
            external_detections: Vec::new(),
        }
    }

    /// Create a new violation with a file-relative span
    #[must_use]
    pub fn from_file_span(message: impl Into<Cow<'static, str>>, span: FileSpan) -> Self {
        Self {
            message: message.into(),
            span: LintSpan::File(span),
            primary_label: None,
            extra_labels: Vec::new(),
            external_detections: Vec::new(),
        }
    }

    /// Add an external detection (for errors in stdlib, imported modules, etc.)
    #[must_use]
    pub fn with_external_detection(mut self, detection: ExternalDetection) -> Self {
        self.external_detections.push(detection);
        self
    }

    #[must_use]
    pub fn with_primary_label(mut self, label: impl Into<Cow<'static, str>>) -> Self {
        self.primary_label = Some(label.into());
        self
    }

    #[must_use]
    pub fn with_extra_label(mut self, label: impl Into<Cow<'static, str>>, span: Span) -> Self {
        self.extra_labels
            .push((LintSpan::from(span), Some(label.into().to_string())));
        self
    }

    #[must_use]
    pub fn with_extra_span(mut self, span: Span) -> Self {
        self.extra_labels.push((LintSpan::from(span), None));
        self
    }
}

/// A lint violation with its full diagnostic information (after fix is
/// attached).
///
/// This is the final form of a violation, constructed by the engine from a
/// `Detection` plus an optional `Fix`. Rules cannot construct this
/// type directly - they return `Detection` from `detect()`.
#[derive(Debug, Clone)]
pub struct Violation {
    pub rule_id: Option<Cow<'static, str>>,
    pub lint_level: LintLevel,
    pub message: Cow<'static, str>,
    pub span: LintSpan,
    pub primary_label: Option<Cow<'static, str>>,
    pub extra_labels: Vec<(LintSpan, Option<String>)>,
    pub help: Option<String>,
    pub fix: Option<Fix>,
    pub(crate) file: Option<SourceFile>,
    pub(crate) source: Option<Cow<'static, str>>,
    pub doc_url: Option<&'static str>,
    /// Related detections in external files
    pub external_detections: Vec<ExternalDetection>,
}

impl Violation {
    pub(crate) fn from_detected(
        detected: Detection,
        fix: Option<Fix>,
        help: impl Into<Option<&'static str>>,
    ) -> Self {
        Self {
            rule_id: None,
            lint_level: LintLevel::default(),
            message: detected.message,
            span: detected.span,
            primary_label: detected.primary_label,
            extra_labels: detected.extra_labels,
            help: help.into().map(ToString::to_string),
            fix,
            file: None,
            source: None,
            doc_url: None,
            external_detections: detected.external_detections,
        }
    }

    /// Set the rule ID for this violation (used by the engine)
    pub(crate) fn set_rule_id(&mut self, rule_id: &'static str) {
        self.rule_id = Some(Cow::Borrowed(rule_id));
    }

    /// Set the lint level for this violation (used by the engine)
    pub(crate) const fn set_lint_level(&mut self, level: LintLevel) {
        self.lint_level = level;
    }

    /// Set the documentation URL for this violation (used by the engine)
    pub(crate) const fn set_doc_url(&mut self, url: Option<&'static str>) {
        self.doc_url = url;
    }

    /// Get the span as file-relative. Panics if not normalized.
    #[must_use]
    pub fn file_span(&self) -> FileSpan {
        self.span.file_span()
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
            "{:?}({})",
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
        let file_span = self.file_span();
        let span_range = file_span.start..file_span.end;
        let primary = self.primary_label.as_ref().map_or_else(
            || LabeledSpan::underline(span_range.clone()),
            |label| LabeledSpan::new_primary_with_span(Some(label.to_string()), span_range.clone()),
        );
        let extras = self.extra_labels.iter().map(|(span, label)| {
            let file_span = span.file_span();
            LabeledSpan::new_with_span(label.clone(), file_span.start..file_span.end)
        });
        Some(Box::new(once(primary).chain(extras)))
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
