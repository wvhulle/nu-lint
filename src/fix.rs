use std::{collections::HashMap, fmt::Write, fs, path::PathBuf};

use crate::{LintError, violation::Violation};

/// Result of applying fixes to a file
#[derive(Debug)]
pub struct FixResult {
    pub file_path: PathBuf,
    pub original_content: String,
    pub fixed_content: String,
    pub fixes_applied: usize,
}

/// Apply fixes to violations grouped by file
///
/// # Errors
///
/// Returns an error if a file cannot be read or written
pub fn apply_fixes(violations: &[Violation], dry_run: bool) -> Result<Vec<FixResult>, LintError> {
    let violations_by_file = group_violations_by_file(violations);
    let mut results = Vec::new();

    for (file_path, file_violations) in violations_by_file {
        let original_content = fs::read_to_string(&file_path)?;
        let fixed_content = apply_fixes_to_content(&original_content, &file_violations);
        let fixes_applied = count_applicable_fixes(&file_violations);

        if fixes_applied > 0 {
            if !dry_run {
                fs::write(&file_path, &fixed_content)?;
            }

            results.push(FixResult {
                file_path,
                original_content,
                fixed_content,
                fixes_applied,
            });
        }
    }

    Ok(results)
}

/// Group violations by their file path
fn group_violations_by_file(violations: &[Violation]) -> HashMap<PathBuf, Vec<&Violation>> {
    let mut grouped: HashMap<PathBuf, Vec<&Violation>> = HashMap::new();

    for violation in violations {
        if let Some(file) = &violation.file {
            let path = PathBuf::from(file.as_ref());
            grouped.entry(path).or_default().push(violation);
        }
    }

    grouped
}

/// Apply fixes to source code content
fn apply_fixes_to_content(content: &str, violations: &[&Violation]) -> String {
    let content_bytes = content.as_bytes();

    // Collect all replacements from all violations
    let mut replacements = Vec::new();
    for violation in violations {
        if let Some(fix) = &violation.fix {
            replacements.extend(fix.replacements.clone());
        }
    }

    if replacements.is_empty() {
        return content.to_string();
    }

    // Sort replacements by span start in reverse order to apply from end to start
    replacements.sort_by(|a, b| b.span.start.cmp(&a.span.start));

    let mut result = content.to_string();

    for replacement in replacements {
        let start = replacement.span.start;
        let end = replacement.span.end;

        // Validate span bounds
        if start > content_bytes.len() || end > content_bytes.len() || start > end {
            log::warn!(
                "Invalid replacement span: start={}, end={}, content_len={}",
                start,
                end,
                content_bytes.len()
            );
            continue;
        }

        // Apply the replacement
        result.replace_range(start..end, &replacement.new_text);
    }

    result
}

/// Count how many violations have applicable fixes
fn count_applicable_fixes(violations: &[&Violation]) -> usize {
    violations.iter().filter(|v| v.fix.is_some()).count()
}

/// Format fix results for output
#[must_use]
pub fn format_fix_results(results: &[FixResult], dry_run: bool) -> String {
    let mut output = String::new();

    if dry_run {
        output.push_str("The following files would be fixed:\n\n");
    } else {
        output.push_str("Fixed the following files:\n\n");
    }

    for result in results {
        writeln!(
            output,
            "  {} ({} fixes)",
            result.file_path.display(),
            result.fixes_applied
        )
        .unwrap();
    }

    if results.is_empty() {
        output.push_str("  No fixable violations found.\n");
    }

    output
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use nu_protocol::Span;

    use super::*;
    use crate::{
        config::LintLevel,
        violation::{Fix, Replacement, Violation},
    };

    #[test]
    fn test_apply_single_replacement() {
        let content = "let x = 5";
        let replacement = Replacement::new_static(Span::new(4, 5), "y");
        let fix = Fix::new_static("Rename variable", vec![replacement]);

        let violation = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(4, 5),
            suggestion: None,
            fix: Some(fix),
            file: Some(Cow::Borrowed("test.nu")),
        };

        let fixed = apply_fixes_to_content(content, &[&violation]);
        assert_eq!(fixed, "let y = 5");
    }

    #[test]
    fn test_apply_multiple_replacements() {
        let content = "let x = 5; let y = 10";
        let replacements = vec![
            Replacement::new_static(Span::new(4, 5), "a"),
            Replacement::new_static(Span::new(15, 16), "b"),
        ];
        let fix = Fix::new_static("Rename variables", replacements);

        let violation = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(0, 21),
            suggestion: None,
            fix: Some(fix),
            file: Some(Cow::Borrowed("test.nu")),
        };

        let fixed = apply_fixes_to_content(content, &[&violation]);
        assert_eq!(fixed, "let a = 5; let b = 10");
    }

    #[test]
    fn test_multiple_fixes_same_file() {
        // Test that multiple separate fixes to different parts of the same file work
        // correctly
        let content = "let x = 5; let y = 10; let z = 15";

        let fix1 = Fix::new_static(
            "Rename x",
            vec![Replacement::new_static(Span::new(4, 5), "a")],
        );

        let fix2 = Fix::new_static(
            "Rename y",
            vec![Replacement::new_static(Span::new(15, 16), "b")],
        );

        let fix3 = Fix::new_static(
            "Rename z",
            vec![Replacement::new_static(Span::new(27, 28), "c")],
        );

        let violation1 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(4, 5),
            suggestion: None,
            fix: Some(fix1),
            file: Some(Cow::Borrowed("test.nu")),
        };

        let violation2 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(15, 16),
            suggestion: None,
            fix: Some(fix2),
            file: Some(Cow::Borrowed("test.nu")),
        };

        let violation3 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(27, 28),
            suggestion: None,
            fix: Some(fix3),
            file: Some(Cow::Borrowed("test.nu")),
        };

        let fixed = apply_fixes_to_content(content, &[&violation1, &violation2, &violation3]);
        assert_eq!(fixed, "let a = 5; let b = 10; let c = 15");
    }

    #[test]
    fn test_overlapping_fixes_with_different_lengths() {
        // Test replacing text with different length strings
        let content = "let variable_name = 5";

        let fix = Fix::new_static(
            "Shorten name",
            vec![Replacement::new_static(Span::new(4, 17), "x")],
        );

        let violation = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(4, 17),
            suggestion: None,
            fix: Some(fix),
            file: Some(Cow::Borrowed("test.nu")),
        };

        let fixed = apply_fixes_to_content(content, &[&violation]);
        assert_eq!(fixed, "let x = 5");
    }

    #[test]
    fn test_multiple_fixes_different_lengths() {
        // Test multiple fixes where replacements have different lengths
        let content = "let abc = 5; let defgh = 10";

        let fix1 = Fix::new_static(
            "Shorten abc to a",
            vec![Replacement::new_static(Span::new(4, 7), "a")],
        );

        let fix2 = Fix::new_static(
            "Shorten defgh to b",
            vec![Replacement::new_static(Span::new(17, 22), "b")],
        );

        let violation1 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(4, 7),
            suggestion: None,
            fix: Some(fix1),
            file: Some(Cow::Borrowed("test.nu")),
        };

        let violation2 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(17, 22),
            suggestion: None,
            fix: Some(fix2),
            file: Some(Cow::Borrowed("test.nu")),
        };

        let fixed = apply_fixes_to_content(content, &[&violation1, &violation2]);
        assert_eq!(fixed, "let a = 5; let b = 10");
    }

    #[test]
    fn test_fixes_applied_in_reverse_order() {
        // Verify that fixes are applied from end to start to preserve offsets
        let content = "aaaa bbbb cccc dddd";

        // Apply fixes in forward order but they should be processed in reverse
        let fix1 = Fix::new_static(
            "Replace aaaa",
            vec![Replacement::new_static(Span::new(0, 4), "A")],
        );

        let fix2 = Fix::new_static(
            "Replace bbbb",
            vec![Replacement::new_static(Span::new(5, 9), "B")],
        );

        let fix3 = Fix::new_static(
            "Replace cccc",
            vec![Replacement::new_static(Span::new(10, 14), "C")],
        );

        let fix4 = Fix::new_static(
            "Replace dddd",
            vec![Replacement::new_static(Span::new(15, 19), "DDDD")],
        );

        let v1 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(0, 4),
            suggestion: None,
            fix: Some(fix1),
            file: Some(Cow::Borrowed("test.nu")),
        };

        let v2 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(5, 9),
            suggestion: None,
            fix: Some(fix2),
            file: Some(Cow::Borrowed("test.nu")),
        };

        let v3 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(10, 14),
            suggestion: None,
            fix: Some(fix3),
            file: Some(Cow::Borrowed("test.nu")),
        };

        let v4 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(15, 19),
            suggestion: None,
            fix: Some(fix4),
            file: Some(Cow::Borrowed("test.nu")),
        };

        // Pass violations in order, but they should be applied in reverse
        let fixed = apply_fixes_to_content(content, &[&v1, &v2, &v3, &v4]);
        assert_eq!(fixed, "A B C DDDD");
    }

    #[test]
    fn test_no_fixes() {
        let content = "let x = 5";
        let violation = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(4, 5),
            suggestion: None,
            fix: None,
            file: Some(Cow::Borrowed("test.nu")),
        };

        let fixed = apply_fixes_to_content(content, &[&violation]);
        assert_eq!(fixed, content);
    }

    #[test]
    fn test_count_applicable_fixes() {
        let fix = Fix::new_static("Test fix", vec![]);

        let with_fix = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(0, 5),
            suggestion: None,
            fix: Some(fix),
            file: Some(Cow::Borrowed("test.nu")),
        };

        let without_fix = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(0, 5),
            suggestion: None,
            fix: None,
            file: Some(Cow::Borrowed("test.nu")),
        };

        let violations = vec![&with_fix, &without_fix, &with_fix];
        assert_eq!(count_applicable_fixes(&violations), 2);
    }

    #[test]
    fn test_group_violations_by_file() {
        let v1 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(0, 5),
            suggestion: None,
            fix: None,
            file: Some(Cow::Borrowed("file1.nu")),
        };

        let v2 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(0, 5),
            suggestion: None,
            fix: None,
            file: Some(Cow::Borrowed("file2.nu")),
        };

        let v3 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(5, 10),
            suggestion: None,
            fix: None,
            file: Some(Cow::Borrowed("file1.nu")),
        };

        let violations = vec![v1, v2, v3];
        let grouped = group_violations_by_file(&violations);

        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[&PathBuf::from("file1.nu")].len(), 2);
        assert_eq!(grouped[&PathBuf::from("file2.nu")].len(), 1);
    }
}
