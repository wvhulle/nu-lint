use miette::SourceSpan;
use nu_protocol::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Violation {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub suggestion: Option<String>,
    pub fix: Option<Fix>,
    pub file: Option<String>,
}

impl Violation {
    #[must_use]
    pub fn to_source_span(&self) -> SourceSpan {
        SourceSpan::from((self.span.start, self.span.end - self.span.start))
    }
}

#[derive(Debug, Clone)]
pub struct Fix {
    pub description: String,
    pub replacements: Vec<Replacement>,
}

#[derive(Debug, Clone)]
pub struct Replacement {
    pub span: Span,
    pub new_text: String,
}
