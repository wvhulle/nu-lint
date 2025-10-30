use super::rule;
use crate::context::LintContext;

#[test]
fn test_suggestion_for_string_literal() {
    let bad_code = r#"echo "hello world""#;
    let violations =
        LintContext::test_with_parsed_source(bad_code, |context| rule().check(&context));

    assert!(!violations.is_empty(), "Should detect echo usage");
    let suggestion = violations[0].suggestion.as_ref().unwrap();

    // Should show the exact code being used
    assert!(
        suggestion.contains(r#"echo "hello world""#),
        "Should show the exact bad code: {suggestion}"
    );
    // Should suggest removing echo
    assert!(
        suggestion.contains(r#""hello world""#) && suggestion.contains("Good:"),
        "Should suggest using the string directly: {suggestion}"
    );
    // Should also mention print as alternative
    assert!(
        suggestion.contains("print"),
        "Should mention print as alternative: {suggestion}"
    );
}

#[test]
fn test_suggestion_for_variable() {
    let bad_code = r"echo $value";
    let violations =
        LintContext::test_with_parsed_source(bad_code, |context| rule().check(&context));

    assert!(!violations.is_empty(), "Should detect echo usage");
    let suggestion = violations[0].suggestion.as_ref().unwrap();

    // Should show the exact variable being echoed
    assert!(
        suggestion.contains("echo $value"),
        "Should show the exact bad code: {suggestion}"
    );
    assert!(
        suggestion.contains("$value") && suggestion.contains("Good:"),
        "Should suggest using variable directly: {suggestion}"
    );
}

#[test]
fn test_suggestion_for_pipeline() {
    let bad_code = r"echo $var | str upcase";
    let violations =
        LintContext::test_with_parsed_source(bad_code, |context| rule().check(&context));

    assert!(!violations.is_empty(), "Should detect echo in pipeline");
    let suggestion = violations[0].suggestion.as_ref().unwrap();

    // Should show removing echo from the pipeline
    assert!(
        suggestion.contains("$var | str upcase"),
        "Should show pipeline without echo: {suggestion}"
    );
    assert!(
        suggestion.contains("echo $var"),
        "Should show the bad pattern: {suggestion}"
    );
}

#[test]
fn test_suggestion_for_external_echo() {
    let bad_code = r#"^echo "test""#;
    let violations =
        LintContext::test_with_parsed_source(bad_code, |context| rule().check(&context));

    assert!(!violations.is_empty(), "Should detect external echo");
    let suggestion = violations[0].suggestion.as_ref().unwrap();

    // Should mention it's external echo
    assert!(
        suggestion.contains("^echo"),
        "Should mention external echo: {suggestion}"
    );
    // Should suggest alternatives
    assert!(
        suggestion.contains("print") || suggestion.contains(r#""test""#),
        "Should suggest alternatives: {suggestion}"
    );
}

#[test]
fn test_message_explains_why_bad() {
    let bad_code = r"echo $value";
    let violations =
        LintContext::test_with_parsed_source(bad_code, |context| rule().check(&context));

    assert!(!violations.is_empty(), "Should detect echo usage");
    let message = &violations[0].message;

    // Message should explain the problem
    assert!(
        message.to_lowercase().contains("avoid") || message.to_lowercase().contains("don't"),
        "Message should discourage usage: {message}"
    );
}

#[test]
fn test_different_suggestions_for_different_patterns() {
    let test_cases = vec![
        (r#"echo "hello""#, r#""hello""#),
        (r"echo $var", "$var"),
        (r#"^echo "world""#, "print"),
    ];

    for (bad_code, expected_in_suggestion) in test_cases {
        let violations =
            LintContext::test_with_parsed_source(bad_code, |context| rule().check(&context));

        assert!(!violations.is_empty(), "Should detect: {bad_code}");

        let suggestion = violations[0].suggestion.as_ref().unwrap();
        assert!(
            suggestion.contains(expected_in_suggestion),
            "For '{bad_code}', suggestion should contain '{expected_in_suggestion}': {suggestion}"
        );
    }
}

#[test]
fn test_suggestion_shows_actual_code_context() {
    let bad_code = r#"echo "Debug: processing item""#;
    let violations =
        LintContext::test_with_parsed_source(bad_code, |context| rule().check(&context));

    assert!(!violations.is_empty(), "Should detect echo usage");
    let suggestion = violations[0].suggestion.as_ref().unwrap();

    // Should show the actual string being echoed
    assert!(
        suggestion.contains("Debug: processing item") || suggestion.contains("echo \"Debug"),
        "Should show actual code context: {suggestion}"
    );
}

#[test]
fn test_each_violation_has_tailored_suggestion() {
    let bad_code = r#"
echo "first"
echo $var
^echo "third"
"#;
    let violations =
        LintContext::test_with_parsed_source(bad_code, |context| rule().check(&context));

    assert_eq!(violations.len(), 3, "Should detect all three echo uses");

    // Each violation should have a suggestion
    for violation in &violations {
        assert!(
            violation.suggestion.is_some(),
            "Each violation should have a suggestion"
        );
    }

    // Suggestions should be different for different patterns
    let suggestion1 = violations[0].suggestion.as_ref().unwrap();
    let suggestion2 = violations[1].suggestion.as_ref().unwrap();
    let suggestion3 = violations[2].suggestion.as_ref().unwrap();

    // First one has string literal
    assert!(
        suggestion1.contains(r#""first""#),
        "First suggestion should mention the string: {suggestion1}"
    );

    // Second one has variable
    assert!(
        suggestion2.contains("$var"),
        "Second suggestion should mention the variable: {suggestion2}"
    );

    // Third one has external echo
    assert!(
        suggestion3.contains("^echo") || suggestion3.contains("third"),
        "Third suggestion should mention external or the string: {suggestion3}"
    );
}

#[test]
fn test_suggestion_format_consistency() {
    let bad_code = r"echo $value";
    let violations =
        LintContext::test_with_parsed_source(bad_code, |context| rule().check(&context));

    assert!(!violations.is_empty(), "Should detect echo usage");
    let suggestion = violations[0].suggestion.as_ref().unwrap();

    // Should have clear structure
    assert!(
        suggestion.contains("Bad:") && suggestion.contains("Good:"),
        "Should have Bad/Good format: {suggestion}"
    );

    // Should have newlines for readability
    assert!(
        suggestion.contains('\n'),
        "Should have newlines for readability: {suggestion}"
    );
}

#[test]
fn test_suggestion_for_multiple_arguments() {
    let bad_code = r"echo hello world test";
    let violations =
        LintContext::test_with_parsed_source(bad_code, |context| rule().check(&context));

    assert!(
        !violations.is_empty(),
        "Should detect echo with multiple args"
    );
    let suggestion = violations[0].suggestion.as_ref().unwrap();

    // Should show the actual command
    assert!(
        suggestion.contains("echo"),
        "Should show echo command: {suggestion}"
    );

    // Should suggest alternatives
    assert!(
        suggestion.contains("print") || suggestion.contains("directly"),
        "Should suggest alternatives: {suggestion}"
    );
}
