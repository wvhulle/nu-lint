use std::borrow::Cow;

use miette::SourceSpan;
use nu_protocol::Span;

use crate::config::LintLevel;

/// A lint violation with its diagnostic information
///
/// # Display Format
///
/// When displayed to the user, violations appear as:
///
/// ```text
/// file.nu:10:5
/// warn(rule_name)
///
///   ⚠ [message]                  <- Short diagnostic message
///    ╭─[10:5]
/// 10 │ code here
///    ·     ─┬─
///    ·      ╰── [label text]     <- Message (or help if short)
///    ╰────
///
///   help: [help text]            <- Detailed explanation/rationale
///
///   ℹ Available fix: [explanation] <- Fix explanation
///   - old code
///   + new code                   <- From replacements
/// ```
///
/// # Example
///
/// ```rust,ignore
/// Violation::new("prefer_pipeline_input", "Use pipeline input", span)
///     .with_help("Pipeline input enables better composability and streaming performance")
///     .with_fix(Fix::with_explanation(
///         format!("Use $in instead of ${}:\n  {}", param, transformed_code),
///         vec![Replacement::new(def_span, transformed_code)]
///     ))
/// ```
#[derive(Debug, Clone)]
pub struct Violation {
    pub rule_id: Cow<'static, str>,
    pub lint_level: LintLevel,

    /// Short message shown in the warning header
    /// Should be concise, typically < 80 chars
    /// Example: "Use pipeline input instead of parameter"
    pub message: Cow<'static, str>,

    /// Span in source code where the violation occurs
    pub span: Span,

    /// Optional detailed explanation shown in the "help:" section
    /// Use this to explain WHY the code should change or provide rationale
    /// Example: "Pipeline input enables better composability and streaming
    /// performance"
    pub help: Option<Cow<'static, str>>,

    /// Optional automated fix that can be applied
    pub fix: Option<Fix>,

    pub(crate) file: Option<Cow<'static, str>>,

    /// Optional source code content (used for stdin or when file is not
    /// accessible)
    pub(crate) source: Option<Cow<'static, str>>,

    /// Optional URL to official Nushell documentation
    pub doc_url: Option<&'static str>,
}

impl Violation {
    /// Create a new violation
    ///
    /// # Arguments
    ///
    /// * `rule_id` - The lint rule identifier (e.g., "`prefer_pipeline_input`")
    /// * `message` - Short diagnostic message shown in the warning header
    /// * `span` - Location in source code where the violation occurs
    #[must_use]
    pub fn new(rule_id: &'static str, message: impl Into<Cow<'static, str>>, span: Span) -> Self {
        Self {
            rule_id: Cow::Borrowed(rule_id),
            lint_level: LintLevel::Allow, // Placeholder, will be set by engine
            message: message.into(),
            span,
            help: None,
            fix: None,
            file: None,
            source: None,
            doc_url: None,
        }
    }

    /// Add detailed help text explaining why this change should be made
    ///
    /// This appears in the "help:" section of the diagnostic output.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// violation.with_help("Pipeline input enables better composability and streaming performance")
    /// ```
    #[must_use]
    pub fn with_help(mut self, help: impl Into<Cow<'static, str>>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Add an automated fix to this violation
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// violation.with_fix(Fix::with_explanation(
    ///     "Replace with pipeline input version",
    ///     vec![Replacement::new(span, new_code)]
    /// ))
    /// ```
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

    #[must_use]
    pub(crate) fn to_source_span(&self) -> SourceSpan {
        SourceSpan::from((self.span.start, self.span.end - self.span.start))
    }
}

/// An automated fix that can be applied to resolve a violation
///
/// # Display Format
///
/// Fixes are displayed as:
///
/// ```text
/// ℹ Available fix: [explanation]
/// - old code
/// + new code
/// ```
///
/// # Important
///
/// - `explanation`: User-facing text shown in "Available fix:" (can be
///   multi-line)
/// - `replacements[].replacement_text`: Actual code written to the file
///
/// These should be different! The explanation describes the change,
/// the `replacement_text` is the actual code.
///
/// # Example
///
/// ```rust,ignore
/// Fix::with_explanation(
///     format!("Use pipeline input ($in) instead of parameter (${}):\n  {}", param, full_code),
///     vec![Replacement::new(span, actual_code_to_write)]
/// )
/// ```
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
    ///
    /// # Arguments
    ///
    /// * `explanation` - User-facing description (shown in "Available fix:")
    /// * `replacements` - Actual code changes to apply to the file
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// Fix::with_explanation(
    ///     "Replace with is-not-empty",
    ///     vec![Replacement::new(span, "is-not-empty")]
    /// )
    /// ```
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
    /// Span in source code to replace
    pub span: Span,

    /// New text to insert at this location
    /// This is the ACTUAL CODE written to the file when the fix is applied
    pub replacement_text: Cow<'static, str>,
}

impl Replacement {
    /// Create a new code replacement
    ///
    /// # Arguments
    ///
    /// * `span` - Location in source code to replace
    /// * `replacement_text` - Actual code to write (not a description!)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// Replacement::new(param_span, "[]")  // Replace "[x: int]" with "[]"
    /// ```
    #[must_use]
    pub fn new(span: Span, replacement_text: impl Into<Cow<'static, str>>) -> Self {
        Self {
            span,
            replacement_text: replacement_text.into(),
        }
    }
}
