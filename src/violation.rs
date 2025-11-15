use std::borrow::Cow;

use miette::SourceSpan;
use nu_protocol::Span;

use crate::config::LintLevel;

/// A rule violation without severity (created by rules)
#[derive(Debug, Clone)]
pub struct RuleViolation {
    pub rule_id: Cow<'static, str>,
    pub message: Cow<'static, str>,
    pub span: Span,
    pub suggestion: Option<Cow<'static, str>>,
    pub(crate) fix: Option<Fix>,
}

impl RuleViolation {
    /// Create a new rule violation with static strings
    #[must_use]
    pub(crate) const fn new_static(
        rule_id: &'static str,
        message: &'static str,
        span: Span,
    ) -> Self {
        Self {
            rule_id: Cow::Borrowed(rule_id),
            message: Cow::Borrowed(message),
            span,
            suggestion: None,
            fix: None,
        }
    }

    /// Create a new rule violation with a dynamic message
    #[must_use]
    pub(crate) const fn new_dynamic(rule_id: &'static str, message: String, span: Span) -> Self {
        Self {
            rule_id: Cow::Borrowed(rule_id),
            message: Cow::Owned(message),
            span,
            suggestion: None,
            fix: None,
        }
    }

    /// Add a static suggestion to this violation
    #[must_use]
    pub(crate) fn with_suggestion_static(mut self, suggestion: &'static str) -> Self {
        self.suggestion = Some(Cow::Borrowed(suggestion));
        self
    }

    /// Add a dynamic suggestion to this violation
    #[must_use]
    pub(crate) fn with_suggestion_dynamic(mut self, suggestion: String) -> Self {
        self.suggestion = Some(Cow::Owned(suggestion));
        self
    }

    /// Add a fix to this violation
    #[must_use]
    pub(crate) fn with_fix(mut self, fix: Fix) -> Self {
        self.fix = Some(fix);
        self
    }

    /// Convert to a full Violation with severity
    #[must_use]
    pub fn into_violation(self, severity: LintLevel) -> Violation {
        Violation {
            rule_id: self.rule_id,
            lint_level: severity,
            message: self.message,
            span: self.span,
            suggestion: self.suggestion,
            fix: self.fix,
            file: None,
        }
    }
}

/// A complete violation with severity
#[derive(Debug, Clone)]
pub struct Violation {
    pub rule_id: Cow<'static, str>,
    pub lint_level: LintLevel,
    pub message: Cow<'static, str>,
    pub span: Span,
    pub suggestion: Option<Cow<'static, str>>,
    pub(crate) fix: Option<Fix>,
    pub file: Option<Cow<'static, str>>,
}

impl Violation {
    /// Create a new violation with static strings (most common case)
    #[must_use]
    pub const fn new_static(
        rule_id: &'static str,
        lint_level: LintLevel,
        message: &'static str,
        span: Span,
    ) -> Self {
        Self {
            rule_id: Cow::Borrowed(rule_id),
            lint_level,
            message: Cow::Borrowed(message),
            span,
            suggestion: None,
            fix: None,
            file: None,
        }
    }

    /// Create a new violation with a dynamic message
    #[must_use]
    pub const fn new_dynamic(
        rule_id: &'static str,
        lint_level: LintLevel,
        message: String,
        span: Span,
    ) -> Self {
        Self {
            rule_id: Cow::Borrowed(rule_id),
            lint_level,
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
    pub(crate) fn to_source_span(&self) -> SourceSpan {
        SourceSpan::from((self.span.start, self.span.end - self.span.start))
    }
}

#[derive(Debug, Clone)]
pub struct Fix {
    pub(crate) description: Cow<'static, str>,
    pub(crate) replacements: Vec<Replacement>,
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
    pub(crate) span: Span,
    pub(crate) new_text: Cow<'static, str>,
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
