use std::collections::HashSet;

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

/// Get the line number (0-indexed) for a byte position
fn line_number_at(source: &str, pos: usize) -> usize {
    source
        .get(..pos)
        .map_or(0, |s| s.bytes().filter(|&b| b == b'\n').count())
}

/// Check if a violation should be ignored based on preceding line
pub fn should_ignore(source: &str, violation_start: usize, rule_id: &str) -> bool {
    let violation_line = line_number_at(source, violation_start);

    violation_line
        .checked_sub(1)
        .and_then(|prev_line_num| source.lines().nth(prev_line_num))
        .and_then(parse_ignore_comment)
        .is_some_and(|rules| rules.contains(&rule_id))
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
        assert!(should_ignore(source, 26, "my_rule"));
    }

    #[test]
    fn dont_ignore_other_rule() {
        let source = "# nu-lint-ignore: my_rule\nlet x = 1";
        assert!(!should_ignore(source, 26, "other_rule"));
    }

    #[test]
    fn dont_ignore_without_comment() {
        let source = "let x = 1";
        assert!(!should_ignore(source, 0, "my_rule"));
    }
}
