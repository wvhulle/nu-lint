use std::ops::Range;

/// A span in global coordinate space (from AST, includes stdlib offset)
///
/// All spans obtained from AST nodes (expressions, calls, blocks) are global.
/// These cannot be used directly for slicing the source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalSpan {
    pub start: usize,
    pub end: usize,
}

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintSpan {
    Global(GlobalSpan),
    File(FileSpan),
}

impl GlobalSpan {
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Convert to file-relative span by subtracting the file offset
    #[must_use]
    pub const fn to_file_span(self, file_offset: usize) -> FileSpan {
        FileSpan {
            start: self.start.saturating_sub(file_offset),
            end: self.end.saturating_sub(file_offset),
        }
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
}

impl FileSpan {
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Convert to global span by adding the file offset
    #[must_use]
    pub const fn to_global_span(self, file_offset: usize) -> GlobalSpan {
        GlobalSpan {
            start: self.start + file_offset,
            end: self.end + file_offset,
        }
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
            Self::Global(g) => g.to_file_span(file_offset),
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

impl From<nu_protocol::Span> for GlobalSpan {
    fn from(span: nu_protocol::Span) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

impl From<GlobalSpan> for nu_protocol::Span {
    fn from(span: GlobalSpan) -> Self {
        Self::new(span.start, span.end)
    }
}

impl From<FileSpan> for nu_protocol::Span {
    fn from(span: FileSpan) -> Self {
        Self::new(span.start, span.end)
    }
}

impl From<LintSpan> for nu_protocol::Span {
    fn from(span: LintSpan) -> Self {
        match span {
            LintSpan::Global(g) => Self::new(g.start, g.end),
            LintSpan::File(f) => Self::new(f.start, f.end),
        }
    }
}

impl From<nu_protocol::Span> for LintSpan {
    fn from(span: nu_protocol::Span) -> Self {
        Self::Global(GlobalSpan::from(span))
    }
}

impl From<GlobalSpan> for LintSpan {
    fn from(span: GlobalSpan) -> Self {
        Self::Global(span)
    }
}

impl From<FileSpan> for LintSpan {
    fn from(span: FileSpan) -> Self {
        Self::File(span)
    }
}
