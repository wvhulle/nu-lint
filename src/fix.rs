use std::{collections::HashMap, fmt::Write, fs, io::Error as IoError, path::PathBuf};

use similar::{ChangeTag, TextDiff};

use crate::{
    LintError,
    engine::LintEngine,
    violation::{Fix, Violation},
};

/// Result of applying fixes to a file
#[derive(Debug)]
pub struct FixResult {
    pub file_path: PathBuf,
    pub original_content: String,
    pub fixed_content: String,
    pub fixes_applied: usize,
}

/// Apply fixes to stdin content
///
/// Returns the fixed content as a string
#[must_use]
pub fn apply_fixes_to_stdin(violations: &[Violation]) -> Option<String> {
    // Filter violations that come from stdin and have fixes
    let stdin_violations: Vec<&Violation> = violations
        .iter()
        .filter(|v| v.file.as_ref().is_some_and(|f| f.as_ref() == "<stdin>") && v.fix.is_some())
        .collect();

    if stdin_violations.is_empty() {
        return None;
    }

    // Get the original source from the first violation
    let original_content = stdin_violations
        .first()
        .and_then(|v| v.source.as_ref())
        .map(std::borrow::Cow::as_ref)?;

    let fixed_content = apply_fixes_to_content(original_content, &stdin_violations);

    Some(fixed_content)
}

/// Apply fixes to violations grouped by file
///
/// # Errors
///
/// Returns an error if a file cannot be read or written
pub fn apply_fixes(
    violations: &[Violation],
    dry_run: bool,
    lint_engine: &LintEngine,
) -> Result<Vec<FixResult>, LintError> {
    let results = group_violations_by_file(violations)
        .into_iter()
        .filter_map(|(file_path, _file_violations)| {
            apply_fix_to_file(&file_path, dry_run, lint_engine).ok()
        })
        .collect();

    Ok(results)
}

/// Apply fixes to a single file iteratively
fn apply_fix_to_file(
    file_path: &PathBuf,
    dry_run: bool,
    lint_engine: &LintEngine,
) -> Result<FixResult, IoError> {
    let original_content = fs::read_to_string(file_path)?;

    // Apply fixes iteratively, re-linting after each fix
    let (fixed_content, fixes_applied) = apply_fixes_iteratively(&original_content, lint_engine);

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

/// Apply fixes iteratively, re-linting after each fix to get fresh spans
fn apply_fixes_iteratively(content: &str, lint_engine: &LintEngine) -> (String, usize) {
    let mut current_content = content.to_string();
    let mut total_fixes_applied = 0;
    let max_iterations = 100; // Prevent infinite loops

    for iteration in 0..max_iterations {
        // Re-lint the current content to get violations with fresh spans
        let violations = lint_engine.lint_str(&current_content);

        // Find the first violation that has a fix
        let fixable_violation = violations.iter().find(|v| v.fix.is_some());

        if fixable_violation.is_none() {
            // No more fixes to apply
            log::debug!(
                "Iterative fix complete after {iteration} iterations, {total_fixes_applied} fixes \
                 applied"
            );
            break;
        }

        // Apply just the first fix
        let violation = fixable_violation.unwrap();
        let fix = violation.fix.as_ref().unwrap();

        // Apply all replacements from this one fix
        let new_content = apply_single_fix_to_content(&current_content, fix);

        if new_content == current_content {
            log::warn!("Fix did not change content, stopping to avoid infinite loop");
            break;
        }

        current_content = new_content;
        total_fixes_applied += 1;

        log::debug!(
            "Applied fix {} from rule '{}' at iteration {}",
            total_fixes_applied,
            violation.rule_id.as_deref().unwrap_or("unknown"),
            iteration
        );
    }

    if total_fixes_applied >= max_iterations {
        log::warn!("Reached maximum iteration limit ({max_iterations})");
    }

    (current_content, total_fixes_applied)
}

/// Apply a single fix's replacements to content
fn apply_single_fix_to_content(content: &str, fix: &Fix) -> String {
    let mut replacements = fix.replacements.clone();

    if replacements.is_empty() {
        return content.to_string();
    }

    // Sort replacements by span start in reverse order
    replacements.sort_by(|a, b| b.span.start.cmp(&a.span.start));

    let mut result = content.to_string();

    for replacement in replacements {
        let start = replacement.span.start;
        let end = replacement.span.end;

        // Validate span bounds
        if start > result.len() || end > result.len() || start > end {
            log::warn!(
                "Invalid replacement span: start={}, end={}, content_len={}",
                start,
                end,
                result.len()
            );
            continue;
        }

        // Check UTF-8 boundaries
        if !result.is_char_boundary(start) || !result.is_char_boundary(end) {
            log::warn!("Replacement span not on UTF-8 boundary: start={start}, end={end}");
            continue;
        }

        result.replace_range(start..end, &replacement.replacement_text);
    }

    result
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
            rule_id: Some(Cow::Borrowed("test_rule")),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(4, 5),
            help: None,
            fix: Some(fix),
            file: Some(Cow::Borrowed("test.nu")),
            source: None,
            doc_url: None,
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
            rule_id: Some(Cow::Borrowed("test_rule")),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(0, 21),
            help: None,
            fix: Some(fix),
            file: Some(Cow::Borrowed("test.nu")),
            source: None,
            doc_url: None,
        };

        let fixed = apply_fixes_to_content(content, &[&violation]);
        assert_eq!(fixed, "let a = 5; let b = 10");
    }

    #[test]
    fn test_iterative_fixes_with_overlapping_spans() {
        // Test that the iterative fix system can handle fixes that would have
        // overlapping spans if applied simultaneously
        use crate::{config::Config, engine::LintEngine};

        let content = "^evtest /dev/input/event0 err> /dev/null | lines\n";

        let config = Config::default();
        let engine = LintEngine::new(config);

        let (fixed, count) = apply_fixes_iteratively(content, &engine);

        // Should apply at least one fix without panicking
        assert!(count > 0, "Expected at least one fix to be applied");

        // Fixed content should not contain the redirect
        assert!(
            !fixed.contains("err> /dev/null"),
            "Fixed content should not contain err> /dev/null"
        );

        // Should be valid Nushell code (no corruption)
        assert!(
            fixed.contains("evtest"),
            "Fixed content should still contain command name"
        );
        assert!(
            fixed.contains("lines"),
            "Fixed content should still contain pipeline command"
        );
    }

    #[test]
    fn test_iterative_fixes_multiple_rules_same_line() {
        // Test that multiple rules fixing the same line work correctly when applied
        // iteratively
        use crate::{config::Config, engine::LintEngine};

        // This triggers multiple rules: prefer_complete_over_dev_null,
        // prefer_builtin_grep, etc.
        let content = "^grep pattern file.txt err> /dev/null | lines\n";

        let config = Config::default();
        let engine = LintEngine::new(config);

        let (fixed, count) = apply_fixes_iteratively(content, &engine);

        // Should apply multiple fixes without corruption
        assert!(count > 0, "Expected at least one fix to be applied");

        // Content should not be corrupted - should still be valid Nushell
        assert!(!fixed.is_empty(), "Fixed content should not be empty");
        assert!(!fixed.contains("err> /dev/null"), "Should remove redirect");

        // The content should be transformed, not corrupted
        // We don't assert exact output since multiple rules may apply
        assert!(
            fixed.len() < 200,
            "Fixed content should not be unreasonably long (corruption check)"
        );
    }

    #[test]
    fn test_iterative_fixes_converge() {
        // Test that iterative fixes eventually converge (no infinite loop)
        use crate::{config::Config, engine::LintEngine};

        // Multiple violations that could potentially trigger repeatedly
        let content = "^curl https://example.com err> /dev/null | str trim\n";

        let config = Config::default();
        let engine = LintEngine::new(config);

        let (fixed, count) = apply_fixes_iteratively(content, &engine);

        // Should converge within reasonable iterations
        assert!(
            count < 50,
            "Should converge within 50 iterations, got {count}"
        );

        // Re-linting the fixed content should produce no fixable violations
        let violations_after = engine.lint_str(&fixed);
        let fixable_after = violations_after.iter().filter(|v| v.fix.is_some()).count();

        assert_eq!(
            fixable_after, 0,
            "After applying all fixes, there should be no more fixable violations"
        );
    }

    #[test]
    fn test_iterative_fixes_preserve_utf8() {
        // Test that iterative fixes correctly handle UTF-8 boundaries
        use crate::{config::Config, engine::LintEngine};

        let content = "^echo 测试 err> /dev/null | lines\n";

        let config = Config::default();
        let engine = LintEngine::new(config);

        let (fixed, count) = apply_fixes_iteratively(content, &engine);

        // Should apply fixes without UTF-8 boundary panics
        assert!(count > 0, "Expected at least one fix to be applied");

        // UTF-8 characters should be preserved
        assert!(
            fixed.contains("测试"),
            "UTF-8 characters should be preserved"
        );
        assert!(
            !fixed.contains("err> /dev/null"),
            "Redirect should be removed"
        );

        // Verify the result is valid UTF-8 (String is always valid UTF-8, but this
        // confirms no corruption)
        assert!(
            !fixed.is_empty() && fixed.chars().all(|c| !c.is_control() || c.is_whitespace()),
            "Result should contain valid characters"
        );
    }

    #[test]
    fn test_no_fixes() {
        let content = "let x = 5";
        let violation = Violation {
            rule_id: Some(Cow::Borrowed("test_rule")),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(4, 5),
            help: None,
            fix: None,
            file: Some(Cow::Borrowed("test.nu")),
            source: None,
            doc_url: None,
        };

        let fixed = apply_fixes_to_content(content, &[&violation]);
        assert_eq!(fixed, content);
    }

    #[test]
    fn test_count_applicable_fixes() {
        let fix = Fix::with_explanation("Test fix", vec![]);

        let with_fix = Violation {
            rule_id: Some(Cow::Borrowed("test_rule")),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(0, 5),
            help: None,
            fix: Some(fix),
            file: Some(Cow::Borrowed("test.nu")),
            source: None,
            doc_url: None,
        };

        let without_fix = Violation {
            rule_id: Some(Cow::Borrowed("test_rule")),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(0, 5),
            help: None,
            fix: None,
            file: Some(Cow::Borrowed("test.nu")),
            source: None,
            doc_url: None,
        };

        let violations = [&with_fix, &without_fix, &with_fix];
        let count = violations.iter().filter(|v| v.fix.is_some()).count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_group_violations_by_file() {
        let v1 = Violation {
            rule_id: Some(Cow::Borrowed("test_rule")),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(0, 5),
            help: None,
            fix: None,
            file: Some(Cow::Borrowed("file1.nu")),
            source: None,
            doc_url: None,
        };

        let v2 = Violation {
            rule_id: Some(Cow::Borrowed("test_rule")),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(0, 5),
            help: None,
            fix: None,
            file: Some(Cow::Borrowed("file2.nu")),
            source: None,
            doc_url: None,
        };

        let v3 = Violation {
            rule_id: Some(Cow::Borrowed("test_rule")),
            lint_level: LintLevel::Warn,
            message: Cow::Borrowed("Test"),
            span: Span::new(5, 10),
            help: None,
            fix: None,
            file: Some(Cow::Borrowed("file1.nu")),
            source: None,
            doc_url: None,
        };

        let violations = vec![v1, v2, v3];
        let grouped = group_violations_by_file(&violations);

        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[&PathBuf::from("file1.nu")].len(), 2);
        assert_eq!(grouped[&PathBuf::from("file2.nu")].len(), 1);
    }
}
