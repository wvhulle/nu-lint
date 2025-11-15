use std::borrow::Cow;

use miette::SourceSpan;
use nu_protocol::Span;

use crate::config::LintLevel;

/// A complete violation with severity
#[derive(Debug, Clone)]
pub struct Violation {
    pub rule_id: Cow<'static, str>,
    pub lint_level: LintLevel,
    pub message: Cow<'static, str>,
    pub span: Span,
    pub suggestion: Option<Cow<'static, str>>,
    pub fix: Option<Fix>,
    pub file: Option<Cow<'static, str>>,
}

impl Violation {
    /// Create a new violation with static strings (most common case)
    /// The lint level will be set by the engine based on configuration
    #[must_use]
    pub const fn new_static(rule_id: &'static str, message: &'static str, span: Span) -> Self {
        Self {
            rule_id: Cow::Borrowed(rule_id),
            lint_level: LintLevel::Allow, // Placeholder, will be set by engine
            message: Cow::Borrowed(message),
            span,
            suggestion: None,
            fix: None,
            file: None,
        }
    }

    /// Create a new violation with a dynamic message
    /// The lint level will be set by the engine based on configuration
    #[must_use]
    pub const fn new_dynamic(rule_id: &'static str, message: String, span: Span) -> Self {
        Self {
            rule_id: Cow::Borrowed(rule_id),
            lint_level: LintLevel::Allow, // Placeholder, will be set by engine
            message: Cow::Owned(message),
            span,
            suggestion: None,
            fix: None,
            file: None,
        }
    }

    /// Add a static suggestion to this violation
    #[must_use]
    pub fn with_suggestion_static(mut self, suggestion: &'static str) -> Self {
        self.suggestion = Some(Cow::Borrowed(suggestion));
        self
    }

    /// Add a dynamic suggestion to this violation
    #[must_use]
    pub fn with_suggestion_dynamic(mut self, suggestion: String) -> Self {
        self.suggestion = Some(Cow::Owned(suggestion));
        self
    }

    /// Add a fix to this violation
    #[must_use]
    pub(crate) fn with_fix(mut self, fix: Fix) -> Self {
        self.fix = Some(fix);
        self
    }

    /// Set the lint level for this violation (used by the engine)
    pub(crate) const fn set_lint_level(&mut self, level: LintLevel) {
        self.lint_level = level;
    }

    #[must_use]
    pub(crate) fn to_source_span(&self) -> SourceSpan {
        SourceSpan::from((self.span.start, self.span.end - self.span.start))
    }
}

#[derive(Debug, Clone)]
pub struct Fix {
    pub description: Cow<'static, str>,
    pub replacements: Vec<Replacement>,
}

impl Fix {
    /// Create a fix with a static description
    #[must_use]
    pub const fn new_static(description: &'static str, replacements: Vec<Replacement>) -> Self {
        Self {
            description: Cow::Borrowed(description),
            replacements,
        }
    }

    /// Create a fix with a dynamic description
    #[must_use]
    pub(crate) const fn new_dynamic(description: String, replacements: Vec<Replacement>) -> Self {
        Self {
            description: Cow::Owned(description),
            replacements,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Replacement {
    pub span: Span,
    pub new_text: Cow<'static, str>,
}

impl Replacement {
    /// Create a replacement with static text
    #[must_use]
    pub const fn new_static(span: Span, new_text: &'static str) -> Self {
        Self {
            span,
            new_text: Cow::Borrowed(new_text),
        }
    }

    /// Create a replacement with dynamic text
    #[must_use]
    pub const fn new_dynamic(span: Span, new_text: String) -> Self {
        Self {
            span,
            new_text: Cow::Owned(new_text),
        }
    }
}
