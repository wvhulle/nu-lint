use super::rule;
use crate::LintContext;

// Note: Automatic fixes for this rule would require significant code
// restructuring (from `rm file | ignore` to `do -i { rm file }`), so we only
// provide detection and suggestions. These tests verify the rule correctly
// identifies the patterns.

#[test]
fn test_detect_rm_with_ignore() {
    let bad_code = "rm /tmp/file.txt | ignore";

    LintContext::test_with_parsed_source(bad_code, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(
            violations.len(),
            1,
            "Should detect exactly one 'rm ... | ignore' pattern"
        );

        let violation = &violations[0];
        assert!(
            violation.message.contains("file operations"),
            "Message should mention file operations"
        );
        assert!(
            violation.message.contains("do -i"),
            "Message should suggest do -i"
        );

        // Verify the suggestion contains the specific command
        let suggestion = violation.suggestion.as_ref().unwrap();
        assert!(
            suggestion.contains("rm /tmp/file.txt"),
            "Suggestion should include the actual command"
        );
        assert!(
            suggestion.contains("do -i { rm /tmp/file.txt }"),
            "Suggestion should show the correct do -i replacement"
        );
        assert!(
            suggestion.contains("try { rm /tmp/file.txt } catch"),
            "Suggestion should show the try-catch alternative"
        );

        // This rule provides suggestions but not automatic fixes
        assert!(
            violation.fix.is_none(),
            "Automatic fix would require complex code restructuring"
        );
    });
}

#[test]
fn test_detect_mv_with_ignore() {
    let bad_code = "mv old.txt new.txt | ignore";

    LintContext::test_with_parsed_source(bad_code, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(
            violations.len(),
            1,
            "Should detect exactly one 'mv ... | ignore' pattern"
        );

        let violation = &violations[0];
        assert!(violation.suggestion.is_some(), "Should provide suggestion");

        let suggestion = violation.suggestion.as_ref().unwrap();
        assert!(
            suggestion.contains("mv old.txt new.txt"),
            "Suggestion should include the actual command"
        );
        assert!(
            suggestion.contains("do -i { mv old.txt new.txt }"),
            "Suggestion should show specific do -i replacement"
        );
        assert!(
            suggestion.contains("Instead of:"),
            "Suggestion should have 'Instead of:' label"
        );
        assert!(
            suggestion.contains("Use:"),
            "Suggestion should have 'Use:' label"
        );
    });
}

#[test]
fn test_detect_cp_with_ignore() {
    let bad_code = "cp source.txt dest.txt | ignore";

    LintContext::test_with_parsed_source(bad_code, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(
            violations.len(),
            1,
            "Should detect exactly one 'cp ... | ignore' pattern"
        );

        let suggestion = violations[0].suggestion.as_ref().unwrap();
        assert!(
            suggestion.contains("cp source.txt dest.txt"),
            "Suggestion should include the actual cp command"
        );
        assert!(
            suggestion.contains("do -i { cp source.txt dest.txt }"),
            "Suggestion should show specific replacement"
        );
    });
}

#[test]
fn test_detect_mkdir_with_ignore() {
    let bad_code = "mkdir /tmp/newdir | ignore";

    LintContext::test_with_parsed_source(bad_code, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(
            violations.len(),
            1,
            "Should detect exactly one 'mkdir ... | ignore' pattern"
        );

        let suggestion = violations[0].suggestion.as_ref().unwrap();
        assert!(
            suggestion.contains("mkdir /tmp/newdir"),
            "Suggestion should include the actual mkdir command"
        );
        assert!(
            suggestion.contains("do -i { mkdir /tmp/newdir }"),
            "Suggestion should show specific replacement"
        );
    });
}

#[test]
fn test_detect_multiple_file_operations() {
    let bad_code = r"
rm file1.txt | ignore
mv old.txt new.txt | ignore
cp a.txt b.txt | ignore
";

    LintContext::test_with_parsed_source(bad_code, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(
            violations.len(),
            3,
            "Should detect exactly three file operations with | ignore"
        );

        // Verify each violation has a customized suggestion
        for (i, violation) in violations.iter().enumerate() {
            assert!(
                violation.suggestion.is_some(),
                "Violation {i} should have a suggestion"
            );

            let suggestion = violation.suggestion.as_ref().unwrap();
            assert!(
                suggestion.contains("do -i {"),
                "Violation {i} should suggest do -i"
            );
            assert!(
                suggestion.contains("try {"),
                "Violation {i} should suggest try-catch alternative"
            );
        }

        // Verify suggestions are command-specific
        let suggestions: Vec<_> = violations
            .iter()
            .map(|v| v.suggestion.as_ref().unwrap())
            .collect();

        assert!(
            suggestions[0].contains("rm file1.txt"),
            "First suggestion should be for rm"
        );
        assert!(
            suggestions[1].contains("mv old.txt new.txt"),
            "Second suggestion should be for mv"
        );
        assert!(
            suggestions[2].contains("cp a.txt b.txt"),
            "Third suggestion should be for cp"
        );
    });
}

#[test]
fn test_suggestion_quality() {
    let bad_code = "rm /tmp/file.txt | ignore";

    LintContext::test_with_parsed_source(bad_code, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 1, "Should detect exactly one violation");

        let violation = &violations[0];
        let suggestion = violation.suggestion.as_ref().unwrap();

        // Verify suggestion structure
        assert!(
            suggestion.contains("'| ignore' only discards output, not errors"),
            "Should explain what ignore does"
        );
        assert!(
            suggestion.contains("Instead of:"),
            "Should have 'Instead of:' section"
        );
        assert!(suggestion.contains("Use:"), "Should have 'Use:' section");
        assert!(
            suggestion.contains("Or use try-catch"),
            "Should mention try-catch alternative"
        );

        // Verify suggestion is command-specific
        assert!(
            suggestion.contains("rm /tmp/file.txt | ignore"),
            "Should show the exact problematic code"
        );
        assert!(
            suggestion.contains("do -i { rm /tmp/file.txt }"),
            "Should show exact do -i replacement"
        );
        assert!(
            suggestion.contains("try { rm /tmp/file.txt } catch { print 'failed' }"),
            "Should show exact try-catch replacement"
        );
    });
}
