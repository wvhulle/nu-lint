use super::rule;
use crate::{context::LintContext, log::instrument};

#[test]
fn test_fix_evtest_redirect() {
    instrument();
    let source = r"^evtest $keyboard err> /dev/null | lines";
    rule().assert_detects(source);
    rule().assert_replacement_contains(source, "^evtest $keyboard | complete | get stdout | lines");
    rule().assert_help_contains(source, "complete");
}

#[test]
fn test_fix_curl_redirect() {
    let source = r"^curl https://example.com err> /dev/null | from json";
    rule().assert_detects(source);
    rule().assert_replacement_contains(
        source,
        "^curl https://example.com | complete | get stdout | from json",
    );
    rule().assert_help_contains(source, "exit_code");
}

#[test]
fn test_fix_grep_redirect() {
    let source = r"^grep 'pattern' file.txt err> /dev/null | lines";
    rule().assert_detects(source);
    rule().assert_replacement_contains(
        source,
        "^grep 'pattern' file.txt | complete | get stdout | lines",
    );
    rule().assert_help_contains(source, "stderr");
}

#[test]
fn test_fix_explanation() {
    let source = r"^curl https://example.com err> /dev/null | lines";
    rule().assert_detects(source);
    rule().assert_fix_explanation_contains(source, "Use complete instead of err> /dev/null");
}

#[test]
fn test_multiple_redirects_generate_multiple_fixes() {
    let source = r"
^curl https://api.example.com err> /dev/null | from json
^grep 'pattern' file.txt err> /dev/null | lines
^wget -qO- https://test.com err> /dev/null | str trim
";
    // Verify all three violations are detected
    rule().assert_count(source, 3);

    // Verify the first fix is correct
    rule().assert_replacement_contains(
        source,
        "^curl https://api.example.com | complete | get stdout | from json",
    );

    // Get all violations to verify each has a fix
    let violations =
        LintContext::test_with_parsed_source(source, |context| (rule().check)(&context));

    assert_eq!(violations.len(), 3, "Should detect 3 violations");

    // Verify all violations have fixes
    for (i, violation) in violations.iter().enumerate() {
        assert!(
            violation.fix.is_some(),
            "Violation {} should have a fix",
            i + 1
        );
        let fix = violation.fix.as_ref().unwrap();
        assert!(
            !fix.replacements.is_empty(),
            "Fix {} should have replacements",
            i + 1
        );
        assert!(
            fix.replacements[0]
                .replacement_text
                .contains("| complete | get stdout"),
            "Fix {} should contain '| complete | get stdout', got: {}",
            i + 1,
            fix.replacements[0].replacement_text
        );
    }
}

#[test]
fn test_redirect_with_complete_already_present() {
    let source = r"^ls /sys/class/backlight/ err> /dev/null | complete | get exit_code";
    rule().assert_detects(source);
    rule().assert_help_contains(source, "Remove the 'err> /dev/null' redirect");
    rule().assert_help_contains(source, "already captures stderr");

    // The fix should just remove the redirect, not add another complete
    rule().assert_replacement_contains(
        source,
        "^ls /sys/class/backlight/ | complete | get exit_code",
    );

    // Should NOT contain duplicate complete
    let violations =
        LintContext::test_with_parsed_source(source, |context| (rule().check)(&context));
    let fix_text = &violations[0].fix.as_ref().unwrap().replacements[0].replacement_text;
    assert!(
        !fix_text.contains("complete | get stdout | complete"),
        "Fix should not contain duplicate complete, got: {fix_text}"
    );
}

#[test]
fn test_message_differs_when_complete_present() {
    let without_complete = r"^curl https://example.com err> /dev/null | lines";
    let with_complete = r"^curl https://example.com err> /dev/null | complete | get exit_code";

    let violations_without =
        LintContext::test_with_parsed_source(without_complete, |context| (rule().check)(&context));
    let violations_with =
        LintContext::test_with_parsed_source(with_complete, |context| (rule().check)(&context));

    assert!(
        violations_without[0]
            .message
            .contains("use 'complete' for idiomatic")
    );
    assert!(
        violations_with[0]
            .message
            .contains("already uses 'complete'")
    );
    assert!(violations_with[0].message.contains("redundant"));
}
