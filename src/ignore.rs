use std::collections::{HashMap, HashSet};

fn split_rules(rules_part: &str) -> Vec<&str> {
    rules_part
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parse `# nu-lint-ignore: rule_a, rule_b` from a line
pub fn parse_ignore_comment(line: &str) -> Option<Vec<&str>> {
    line.trim()
        .strip_prefix('#')
        .map(str::trim)
        .and_then(|s| s.strip_prefix("nu-lint-ignore:"))
        .map(split_rules)
}

/// Parse `# nu-lint-ignore-file: rule_a, rule_b` from a line
pub fn parse_file_ignore_comment(line: &str) -> Option<Vec<&str>> {
    line.trim()
        .strip_prefix('#')
        .map(str::trim)
        .and_then(|s| s.strip_prefix("nu-lint-ignore-file:"))
        .map(split_rules)
}

/// Precomputed index of ignore comments for efficient O(1) lookups.
/// Built once per file, maps target line numbers to sets of ignored rule IDs.
pub struct IgnoreIndex {
    /// Map from line number to set of rule IDs to ignore on that line
    ignored_lines: HashMap<usize, HashSet<String>>,
    /// Rule IDs ignored for the whole file (declared via
    /// `# nu-lint-ignore-file:` at the top).
    file_ignored: HashSet<String>,
    /// Byte offset of each line start (for offset-to-line conversion)
    line_offsets: Vec<usize>,
}

impl IgnoreIndex {
    /// Build an ignore index from source code.
    /// Scans for `# nu-lint-ignore:` comments in two forms:
    /// 1. Inline comments on the same line as code
    /// 2. Standalone comments on previous line (skipping attributes and empty
    ///    lines)
    pub fn new(source: &str) -> Self {
        let lines: Vec<&str> = source.lines().collect();
        let mut ignored_lines = HashMap::new();
        let file_ignored = collect_file_ignored(&lines);

        let mut line_offsets = vec![0];
        for (pos, ch) in source.char_indices() {
            if ch == '\n' {
                line_offsets.push(pos + 1);
            }
        }

        for (line_num, line) in lines.iter().enumerate() {
            if let Some(rules) = parse_ignore_comment(line) {
                let rule_set: HashSet<String> = rules.iter().map(|&s| String::from(s)).collect();
                let target = find_target_line(&lines, line_num + 1);
                ignored_lines
                    .entry(target)
                    .or_insert_with(HashSet::new)
                    .extend(rule_set);
            } else if let Some(comment_start) = line.find('#') {
                let comment_part = &line[comment_start..];
                if let Some(rules) = parse_ignore_comment(comment_part) {
                    let rule_set: HashSet<String> =
                        rules.iter().map(|&s| String::from(s)).collect();
                    ignored_lines
                        .entry(line_num)
                        .or_insert_with(HashSet::new)
                        .extend(rule_set);
                }
            }
        }

        Self {
            ignored_lines,
            file_ignored,
            line_offsets,
        }
    }

    /// Check if a violation at the given byte offset should be ignored for a
    /// rule.
    pub fn should_ignore(&self, byte_offset: usize, rule_id: &str) -> bool {
        if self.file_ignored.contains(rule_id) {
            return true;
        }
        let line = self.offset_to_line(byte_offset);
        self.ignored_lines
            .get(&line)
            .is_some_and(|rules| rules.contains(rule_id))
    }

    /// Convert a byte offset to a line number (0-indexed)
    fn offset_to_line(&self, offset: usize) -> usize {
        self.line_offsets
            .partition_point(|&start| start <= offset)
            .saturating_sub(1)
    }
}

/// The header consists of blank lines and `#`-prefixed comments (including a
/// `#!` shebang). The first line whose first non-whitespace byte is anything
/// else terminates the header.
pub fn is_header_line(line: &str) -> bool {
    match line.bytes().find(|b| !b.is_ascii_whitespace()) {
        None => true,
        Some(b'#') => true,
        _ => false,
    }
}

/// Collect all rule IDs declared in `# nu-lint-ignore-file:` comments at the
/// top of the file.
fn collect_file_ignored(lines: &[&str]) -> HashSet<String> {
    lines
        .iter()
        .take_while(|line| is_header_line(line))
        .filter_map(|line| parse_file_ignore_comment(line))
        .flat_map(|rules| rules.into_iter().map(String::from))
        .collect()
}

/// Find the target line for an ignore comment, skipping attributes and empty
/// lines.
fn find_target_line(lines: &[&str], start: usize) -> usize {
    lines
        .iter()
        .enumerate()
        .skip(start)
        .find(|(_, line)| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('@')
        })
        .map_or(start, |(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_rule() {
        assert_eq!(
            parse_ignore_comment("# nu-lint-ignore: my_rule"),
            Some(vec!["my_rule"])
        );
    }

    #[test]
    fn parse_multiple_rules() {
        assert_eq!(
            parse_ignore_comment("# nu-lint-ignore: rule_a, rule_b"),
            Some(vec!["rule_a", "rule_b"])
        );
    }

    #[test]
    fn parse_regular_comment_returns_none() {
        assert_eq!(parse_ignore_comment("# regular comment"), None);
    }

    #[test]
    fn ignore_matching_rule() {
        let source = "# nu-lint-ignore: my_rule\nlet x = 1";
        let index = IgnoreIndex::new(source);
        assert!(index.should_ignore(26, "my_rule"));
    }

    #[test]
    fn dont_ignore_other_rule() {
        let source = "# nu-lint-ignore: my_rule\nlet x = 1";
        let index = IgnoreIndex::new(source);
        assert!(!index.should_ignore(26, "other_rule"));
    }

    #[test]
    fn dont_ignore_without_comment() {
        let source = "let x = 1";
        let index = IgnoreIndex::new(source);
        assert!(!index.should_ignore(0, "my_rule"));
    }

    #[test]
    fn ignore_with_attributes() {
        let source = "# nu-lint-ignore: my_rule\n@search-terms 'test'\ndef my-cmd [] {}";
        let index = IgnoreIndex::new(source);
        // The def is on line 2 (0-indexed), byte offset is after the first two lines
        let def_offset = source.find("def").unwrap();
        assert!(index.should_ignore(def_offset, "my_rule"));
    }

    #[test]
    fn ignore_with_multiple_attributes() {
        let source =
            "# nu-lint-ignore: my_rule\n@category 'test'\n@search-terms 'a'\ndef my-cmd [] {}";
        let index = IgnoreIndex::new(source);
        let def_offset = source.find("def").unwrap();
        assert!(index.should_ignore(def_offset, "my_rule"));
    }

    #[test]
    fn ignore_inline_comment() {
        let source = "let x = 1 # nu-lint-ignore: my_rule";
        let index = IgnoreIndex::new(source);
        // Violation at the beginning of the line should be ignored
        assert!(index.should_ignore(0, "my_rule"));
        // Violation in the middle should also be ignored
        assert!(index.should_ignore(4, "my_rule"));
    }

    #[test]
    fn file_ignore_applies_everywhere() {
        let source = "# nu-lint-ignore-file: my_rule\nlet x = 1\nlet y = 2";
        let index = IgnoreIndex::new(source);
        let x_off = source.find("let x").unwrap();
        let y_off = source.find("let y").unwrap();
        assert!(index.should_ignore(x_off, "my_rule"));
        assert!(index.should_ignore(y_off, "my_rule"));
        assert!(!index.should_ignore(x_off, "other"));
    }

    #[test]
    fn file_ignore_below_shebang() {
        let source = "#!/usr/bin/env nu\n# nu-lint-ignore-file: r\nlet x = 1";
        let index = IgnoreIndex::new(source);
        let off = source.find("let x").unwrap();
        assert!(index.should_ignore(off, "r"));
    }

    #[test]
    fn file_ignore_not_picked_up_mid_file() {
        let source = "let x = 1\n# nu-lint-ignore-file: r\nlet y = 2";
        let index = IgnoreIndex::new(source);
        let off = source.find("let y").unwrap();
        assert!(!index.should_ignore(off, "r"));
    }

    #[test]
    fn file_ignore_multiple_rules() {
        let source = "# nu-lint-ignore-file: a, b\nlet x = 1";
        let index = IgnoreIndex::new(source);
        let off = source.find("let x").unwrap();
        assert!(index.should_ignore(off, "a"));
        assert!(index.should_ignore(off, "b"));
        assert!(!index.should_ignore(off, "c"));
    }

    #[test]
    fn ignore_inline_with_multiple_rules() {
        let source = "let x = 1 # nu-lint-ignore: rule_a, rule_b";
        let index = IgnoreIndex::new(source);
        assert!(index.should_ignore(0, "rule_a"));
        assert!(index.should_ignore(0, "rule_b"));
        assert!(!index.should_ignore(0, "rule_c"));
    }
}
