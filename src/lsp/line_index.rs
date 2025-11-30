use lsp_types::{Position, Range};

#[allow(
    clippy::cast_possible_truncation,
    reason = "LSP uses u32, files larger than 4GB lines/columns are unrealistic"
)]
#[must_use]
const fn to_lsp_u32(value: usize) -> u32 {
    if value > u32::MAX as usize {
        u32::MAX
    } else {
        value as u32
    }
}

/// Pre-computed line offset index for efficient byte offset to line/column
/// conversion. Uses binary search for O(log n) lookups instead of O(n)
/// iteration.
pub struct LineIndex {
    /// Byte offsets where each line starts. `line_offsets[0]` = 0 (first line
    /// starts at byte 0).
    line_offsets: Vec<usize>,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let mut line_offsets = vec![0];
        for (pos, ch) in source.char_indices() {
            if ch == '\n' {
                line_offsets.push(pos + 1);
            }
        }
        Self { line_offsets }
    }

    /// Convert a byte offset to LSP Position (0-indexed line and column).
    pub fn offset_to_position(&self, offset: usize, source: &str) -> Position {
        let line = self
            .line_offsets
            .partition_point(|&line_start| line_start <= offset)
            .saturating_sub(1);

        let line_start = self.line_offsets.get(line).copied().unwrap_or(0);
        let end_offset = offset.min(source.len());
        let column = source
            .get(line_start..end_offset)
            .map_or(0, |s| s.chars().count());

        Position {
            line: to_lsp_u32(line),
            character: to_lsp_u32(column),
        }
    }

    pub fn span_to_range(&self, source: &str, start: usize, end: usize) -> Range {
        Range {
            start: self.offset_to_position(start, source),
            end: self.offset_to_position(end, source),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_line() {
        let source = "hello world";
        let index = LineIndex::new(source);
        assert_eq!(index.line_offsets, vec![0]);
    }

    #[test]
    fn multiple_lines() {
        let source = "line1\nline2\nline3";
        let index = LineIndex::new(source);
        assert_eq!(index.line_offsets, vec![0, 6, 12]);
    }

    #[test]
    fn empty_source() {
        let source = "";
        let index = LineIndex::new(source);
        assert_eq!(index.line_offsets, vec![0]);
    }

    #[test]
    fn trailing_newline() {
        let source = "line1\n";
        let index = LineIndex::new(source);
        assert_eq!(index.line_offsets, vec![0, 6]);
    }

    #[test]
    fn offset_to_position_start() {
        let source = "hello";
        let index = LineIndex::new(source);
        let pos = index.offset_to_position(0, source);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 0);
    }

    #[test]
    fn offset_to_position_middle_of_line() {
        let source = "hello world";
        let index = LineIndex::new(source);
        let pos = index.offset_to_position(6, source);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 6);
    }

    #[test]
    fn offset_to_position_after_newline() {
        let source = "hello\nworld";
        let index = LineIndex::new(source);
        let pos = index.offset_to_position(6, source);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.character, 0);
    }

    #[test]
    fn offset_to_position_multiple_lines() {
        let source = "line1\nline2\nline3";
        let index = LineIndex::new(source);

        let pos = index.offset_to_position(12, source);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.character, 0);

        let pos = index.offset_to_position(14, source);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.character, 2);
    }

    #[test]
    fn offset_to_position_at_end_of_file() {
        let source = "hello";
        let index = LineIndex::new(source);
        let pos = index.offset_to_position(5, source);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 5);
    }

    #[test]
    fn offset_to_position_beyond_end() {
        let source = "hello";
        let index = LineIndex::new(source);
        let pos = index.offset_to_position(100, source);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 5);
    }

    #[test]
    fn span_to_range_single_line() {
        let source = "let x = 5";
        let index = LineIndex::new(source);
        let range = index.span_to_range(source, 4, 5);
        assert_eq!(range.start.line, 0);
        assert_eq!(range.start.character, 4);
        assert_eq!(range.end.line, 0);
        assert_eq!(range.end.character, 5);
    }

    #[test]
    fn span_to_range_multiline() {
        let source = "def foo [] {\n    bar\n}";
        let index = LineIndex::new(source);
        let range = index.span_to_range(source, 0, 22);
        assert_eq!(range.start.line, 0);
        assert_eq!(range.start.character, 0);
        assert_eq!(range.end.line, 2);
    }
}
