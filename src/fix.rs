use std::{collections::HashMap, fmt::Write, fs, io::Error as IoError, path::PathBuf};

use similar::{ChangeTag, TextDiff};

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
    let results = group_violations_by_file(violations)
        .into_iter()
        .filter_map(|(file_path, file_violations)| {
            apply_fix_to_file(&file_path, &file_violations, dry_run).ok()
        })
        .collect();

    Ok(results)
}

/// Apply fixes to a single file
fn apply_fix_to_file(
    file_path: &PathBuf,
    file_violations: &[&Violation],
    dry_run: bool,
) -> Result<FixResult, IoError> {
    let original_content = fs::read_to_string(file_path)?;
    let fixed_content = apply_fixes_to_content(&original_content, file_violations);
    let fixes_applied = count_applicable_fixes(file_violations);

    log::debug!(
        "File: {}, Fixes: {}, Original len: {}, Fixed len: {}",
        file_path.display(),
        fixes_applied,
        original_content.len(),
        fixed_content.len()
    );

    if fixes_applied == 0 {
        return Err(IoError::other("No fixes to apply"));
    }

    if !dry_run {
        fs::write(file_path, &fixed_content)?;
    }

    Ok(FixResult {
        file_path: file_path.clone(),
        original_content,
        fixed_content,
        fixes_applied,
    })
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
    // This ensures that earlier positions remain valid as we modify the string
    replacements.sort_by(|a, b| b.span.start.cmp(&a.span.start));

    // Deduplicate replacements with identical spans
    // This prevents applying the same fix multiple times
    replacements.dedup_by(|a, b| a.span.start == b.span.start && a.span.end == b.span.end);

    let mut result = content.to_string();
    let content_bytes = content.as_bytes();

    for replacement in replacements {
        let start = replacement.span.start;
        let end = replacement.span.end;

        // Validate span bounds against original content
        if start > content_bytes.len() || end > content_bytes.len() || start > end {
            log::warn!(
                "Invalid replacement span: start={}, end={}, content_len={}",
                start,
                end,
                content_bytes.len()
            );
            continue;
        }

        // Apply the replacement to the result string
        result.replace_range(start..end, &replacement.replacement_text);
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

    if results.is_empty() {
        output.push_str("No fixable violations found.\n");
        return output;
    }

    if dry_run {
        writeln!(
            output,
            "The following changes would be applied ({} file{}):\n",
            results.len(),
            if results.len() == 1 { "" } else { "s" }
        )
        .unwrap();

        for result in results {
            writeln!(output, "File: {}", result.file_path.display()).unwrap();
            writeln!(output, "Fixes to apply: {}\n", result.fixes_applied).unwrap();

            // Generate and display unified diff
            let diff = generate_diff(
                &result.original_content,
                &result.fixed_content,
                &result.file_path,
            );
            output.push_str(&diff);
            output.push('\n');
        }
    } else {
        writeln!(
            output,
            "Fixed {} file{}:\n",
            results.len(),
            if results.len() == 1 { "" } else { "s" }
        )
        .unwrap();

        for result in results {
            writeln!(
                output,
                "  {} ({} fix{})",
                result.file_path.display(),
                result.fixes_applied,
                if result.fixes_applied == 1 { "" } else { "es" }
            )
            .unwrap();
        }
    }

    output
}

/// Generate a unified diff between original and fixed content
fn generate_diff(original: &str, fixed: &str, _file_path: &PathBuf) -> String {
    let diff = TextDiff::from_lines(original, fixed);
    let mut output = String::new();

    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            writeln!(output, "{:-^1$}", "-", 80).unwrap();
        }

        for op in group {
            write_diff_changes(&diff, op, &mut output);
        }
    }

    if output.is_empty() {
        "No changes\n".to_string()
    } else {
        output
    }
}

/// Write diff changes for a single operation
fn write_diff_changes(diff: &TextDiff<'_, '_, '_, str>, op: &similar::DiffOp, output: &mut String) {
    for change in diff.iter_changes(op) {
        let (sign, style) = match change.tag() {
            ChangeTag::Delete => ("-", "\x1b[31m"), // Red
            ChangeTag::Insert => ("+", "\x1b[32m"), // Green
            ChangeTag::Equal => (" ", ""),
        };

        let line_number = change
            .old_index()
            .map_or("    ".to_string(), |idx| (idx + 1).to_string());

        write!(output, "{style}{sign}{line_number:>4} {}", change.value()).unwrap();

        if !style.is_empty() {
            output.push_str("\x1b[0m"); // Reset color
        }
    }
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
        let replacement = Replacement::new(Span::new(4, 5), "y");
        let fix = Fix::with_explanation("Rename variable", vec![replacement]);

        let violation = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(4, 5),
            help: None,
            fix: Some(fix),
            file: Some(Cow::Borrowed("test.nu")),
            source: None,
        };

        let fixed = apply_fixes_to_content(content, &[&violation]);
        assert_eq!(fixed, "let y = 5");
    }

    #[test]
    fn test_apply_multiple_replacements() {
        let content = "let x = 5; let y = 10";
        let replacements = vec![
            Replacement::new(Span::new(4, 5), "a"),
            Replacement::new(Span::new(15, 16), "b"),
        ];
        let fix = Fix::with_explanation("Rename variables", replacements);

        let violation = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(0, 21),
            help: None,
            fix: Some(fix),
            file: Some(Cow::Borrowed("test.nu")),
            source: None,
        };

        let fixed = apply_fixes_to_content(content, &[&violation]);
        assert_eq!(fixed, "let a = 5; let b = 10");
    }

    #[test]
    fn test_multiple_fixes_same_file() {
        // Test that multiple separate fixes to different parts of the same file work
        // correctly
        let content = "let x = 5; let y = 10; let z = 15";

        let fix1 = Fix::with_explanation("Rename x", vec![Replacement::new(Span::new(4, 5), "a")]);

        let fix2 =
            Fix::with_explanation("Rename y", vec![Replacement::new(Span::new(15, 16), "b")]);

        let fix3 =
            Fix::with_explanation("Rename z", vec![Replacement::new(Span::new(27, 28), "c")]);

        let violation1 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(4, 5),
            help: None,
            fix: Some(fix1),
            file: Some(Cow::Borrowed("test.nu")),
            source: None,
        };

        let violation2 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(15, 16),
            help: None,
            fix: Some(fix2),
            file: Some(Cow::Borrowed("test.nu")),
            source: None,
        };

        let violation3 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(27, 28),
            help: None,
            fix: Some(fix3),
            file: Some(Cow::Borrowed("test.nu")),
            source: None,
        };

        let fixed = apply_fixes_to_content(content, &[&violation1, &violation2, &violation3]);
        assert_eq!(fixed, "let a = 5; let b = 10; let c = 15");
    }

    #[test]
    fn test_overlapping_fixes_with_different_lengths() {
        // Test replacing text with different length strings
        let content = "let variable_name = 5";

        let fix = Fix::with_explanation(
            "Shorten name",
            vec![Replacement::new(Span::new(4, 17), "x")],
        );

        let violation = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(4, 17),
            help: None,
            fix: Some(fix),
            file: Some(Cow::Borrowed("test.nu")),
            source: None,
        };

        let fixed = apply_fixes_to_content(content, &[&violation]);
        assert_eq!(fixed, "let x = 5");
    }

    #[test]
    fn test_multiple_fixes_different_lengths() {
        // Test multiple fixes where replacements have different lengths
        let content = "let abc = 5; let defgh = 10";

        let fix1 = Fix::with_explanation(
            "Shorten abc to a",
            vec![Replacement::new(Span::new(4, 7), "a")],
        );

        let fix2 = Fix::with_explanation(
            "Shorten defgh to b",
            vec![Replacement::new(Span::new(17, 22), "b")],
        );

        let violation1 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(4, 7),
            help: None,
            fix: Some(fix1),
            file: Some(Cow::Borrowed("test.nu")),
            source: None,
        };

        let violation2 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(17, 22),
            help: None,
            fix: Some(fix2),
            file: Some(Cow::Borrowed("test.nu")),
            source: None,
        };

        let fixed = apply_fixes_to_content(content, &[&violation1, &violation2]);
        assert_eq!(fixed, "let a = 5; let b = 10");
    }

    #[test]
    fn test_fixes_applied_in_reverse_order() {
        // Verify that fixes are applied from end to start to preserve offsets
        let content = "aaaa bbbb cccc dddd";

        // Apply fixes in forward order but they should be processed in reverse
        let fix1 =
            Fix::with_explanation("Replace aaaa", vec![Replacement::new(Span::new(0, 4), "A")]);

        let fix2 =
            Fix::with_explanation("Replace bbbb", vec![Replacement::new(Span::new(5, 9), "B")]);

        let fix3 = Fix::with_explanation(
            "Replace cccc",
            vec![Replacement::new(Span::new(10, 14), "C")],
        );

        let fix4 = Fix::with_explanation(
            "Replace dddd",
            vec![Replacement::new(Span::new(15, 19), "DDDD")],
        );

        let v1 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(0, 4),
            help: None,
            fix: Some(fix1),
            file: Some(Cow::Borrowed("test.nu")),
            source: None,
        };

        let v2 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(5, 9),
            help: None,
            fix: Some(fix2),
            file: Some(Cow::Borrowed("test.nu")),
            source: None,
        };

        let v3 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(10, 14),
            help: None,
            fix: Some(fix3),
            file: Some(Cow::Borrowed("test.nu")),
            source: None,
        };

        let v4 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(15, 19),
            help: None,
            fix: Some(fix4),
            file: Some(Cow::Borrowed("test.nu")),
            source: None,
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
            help: None,
            fix: None,
            file: Some(Cow::Borrowed("test.nu")),
            source: None,
        };

        let fixed = apply_fixes_to_content(content, &[&violation]);
        assert_eq!(fixed, content);
    }

    #[test]
    fn test_count_applicable_fixes() {
        let fix = Fix::with_explanation("Test fix", vec![]);

        let with_fix = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(0, 5),
            help: None,
            fix: Some(fix),
            file: Some(Cow::Borrowed("test.nu")),
            source: None,
        };

        let without_fix = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(0, 5),
            help: None,
            fix: None,
            file: Some(Cow::Borrowed("test.nu")),
            source: None,
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
            help: None,
            fix: None,
            file: Some(Cow::Borrowed("file1.nu")),
            source: None,
        };

        let v2 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(0, 5),
            help: None,
            fix: None,
            file: Some(Cow::Borrowed("file2.nu")),
            source: None,
        };

        let v3 = Violation {
            rule_id: Cow::Borrowed("test_rule"),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(5, 10),
            help: None,
            fix: None,
            file: Some(Cow::Borrowed("file1.nu")),
            source: None,
        };

        let violations = vec![v1, v2, v3];
        let grouped = group_violations_by_file(&violations);

        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[&PathBuf::from("file1.nu")].len(), 2);
        assert_eq!(grouped[&PathBuf::from("file2.nu")].len(), 1);
    }
}
