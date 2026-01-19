use std::collections::{HashMap, HashSet};

use crate::{
    rules::USED_RULES,
    span::FileSpan,
    violation::{Detection, Violation},
};

/// Parse `# nu-lint-ignore: rule_a, rule_b` from a line
pub fn parse_ignore_comment(line: &str) -> Option<Vec<&str>> {
    line.trim()
        .strip_prefix('#')
        .map(str::trim)
        .and_then(|s| s.strip_prefix("nu-lint-ignore:"))
        .map(|rules_part| {
            rules_part
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect()
        })
}

/// Precomputed index of ignore comments for efficient O(1) lookups.
/// Built once per file, maps target line numbers to sets of ignored rule IDs.
pub struct IgnoreIndex {
    /// Map from line number to set of rule IDs to ignore on that line
    ignored_lines: HashMap<usize, HashSet<String>>,
    /// Byte offset of each line start (for offset-to-line conversion)
    line_offsets: Vec<usize>,
}

impl IgnoreIndex {
    /// Build an ignore index from source code.
    /// Scans for `# nu-lint-ignore:` comments and maps them to their target
    /// lines, skipping over attribute lines (`@...`) and empty lines.
    pub fn new(source: &str) -> Self {
        let lines: Vec<&str> = source.lines().collect();
        let mut ignored_lines = HashMap::new();

        // Build line offsets for offset-to-line conversion
        let mut line_offsets = vec![0];
        for (pos, ch) in source.char_indices() {
            if ch == '\n' {
                line_offsets.push(pos + 1);
            }
        }

        for (line_num, line) in lines.iter().enumerate() {
            if let Some(rules) = parse_ignore_comment(line) {
                // Find the target line (skip attributes and empty lines)
                let target = find_target_line(&lines, line_num + 1);
                let rule_set: HashSet<String> = rules.into_iter().map(String::from).collect();
                ignored_lines
                    .entry(target)
                    .or_insert_with(HashSet::new)
                    .extend(rule_set);
            }
        }

        Self {
            ignored_lines,
            line_offsets,
        }
    }

    /// Check if a violation at the given byte offset should be ignored for a
    /// rule.
    pub fn should_ignore(&self, byte_offset: usize, rule_id: &str) -> bool {
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

/// Find the target line for an ignore comment, skipping attributes and empty
/// lines.
fn find_target_line(lines: &[&str], start: usize) -> usize {
    for (i, line) in lines.iter().enumerate().skip(start) {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('@') {
            return i;
        }
    }
    start
}

/// Validate ignore comments and return warnings for unknown rule IDs
pub fn validate_ignores(source: &str) -> Vec<Violation> {
    let known: HashSet<&str> = USED_RULES.iter().map(|r| r.id()).collect();

    source
        .lines()
        .enumerate()
        .filter_map(|(line_num, line)| {
            parse_ignore_comment(line).map(|rules| (line_num, line, rules))
        })
        .flat_map(|(line_num, line, rules)| {
            let start: usize = source.lines().take(line_num).map(|l| l.len() + 1).sum();
            let end = start + line.len();

            rules
                .into_iter()
                .filter(|rule_id| !known.contains(rule_id))
                .map(move |rule_id| {
                    let mut v = Violation::from_detected(
                        Detection::from_file_span(
                            format!("Unknown rule '{rule_id}' in ignore comment"),
                            FileSpan::new(start, end),
                        ),
                        None,
                        None,
                    );
                    v.set_rule_id("unknown_ignore_rule");
                    v
                })
        })
        .collect()
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
}
