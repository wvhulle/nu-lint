use std::ops::Range;

/// A span relative to the current file being linted (starts at 0)
///
/// Use for:
/// - Creating `Replacement` spans
/// - Regex match positions on `whole_source()`
/// - Manual line/column calculations
/// - Slicing source strings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileSpan {
    pub start: usize,
    pub end: usize,
}

/// A span that can be either global (AST) or file-relative
///
/// Rules return this type, and the engine normalizes all to `FileSpan` before
/// output.
///
/// Global spans are from AST nodes (`nu_protocol::Span`) and include stdlib
/// offset. File spans are relative to the current file being linted (starts at
/// 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintSpan {
    Global(nu_protocol::Span),
    File(FileSpan),
}

impl FileSpan {
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Convert to global span by adding the file offset
    #[must_use]
    pub fn to_global_span(self, file_offset: usize) -> nu_protocol::Span {
        nu_protocol::Span::new(self.start + file_offset, self.end + file_offset)
    }

    /// Create a span that encompasses both self and other
    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    #[must_use]
    pub const fn as_range(&self) -> Range<usize> {
        self.start..self.end
    }
}

impl LintSpan {
    /// Convert to file-relative span, normalizing if needed
    #[must_use]
    pub const fn to_file_span(self, file_offset: usize) -> FileSpan {
        match self {
            Self::Global(g) => FileSpan {
                start: g.start.saturating_sub(file_offset),
                end: g.end.saturating_sub(file_offset),
            },
            Self::File(f) => f,
        }
    }

    /// Get the file-relative span, panicking if not normalized.
    ///
    /// This should only be called after `normalize_spans()` has been invoked.
    #[must_use]
    pub fn file_span(&self) -> FileSpan {
        match self {
            Self::File(f) => *f,
            Self::Global(_) => panic!("Span not normalized - call normalize_spans first"),
        }
    }
}

impl From<nu_protocol::Span> for LintSpan {
    fn from(span: nu_protocol::Span) -> Self {
        Self::Global(span)
    }
}

impl From<FileSpan> for LintSpan {
    fn from(span: FileSpan) -> Self {
        Self::File(span)
    }
}

impl From<FileSpan> for nu_protocol::Span {
    fn from(span: FileSpan) -> Self {
        Self::new(span.start, span.end)
    }
}
