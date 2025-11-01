use crate::{config::Config, engine::LintEngine};

// These tests verify that parse error messages from Nushell's parser
// are properly propagated as violation messages and suggestions.

#[test]
fn test_unclosed_parenthesis_message_propagation() {
    let engine = LintEngine::new(Config::default());
    let code = "let x = (";
    let violations = engine.lint_source(code, None);

    let parse_errors: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id.as_ref() == "nu_parse_error")
        .collect();

    assert!(!parse_errors.is_empty(), "Expected parse error");

    // Parse error message should be meaningful and describe the issue
    let has_meaningful_message = parse_errors.iter().any(|error| {
        let msg = error.message.to_string().to_lowercase();
        msg.contains("unclosed")
            || msg.contains("delimiter")
            || msg.contains("expected")
            || msg.contains("end of code")
            || msg.contains(')')
    });

    assert!(
        has_meaningful_message,
        "Parse error should have a meaningful message describing the unclosed parenthesis. Got: \
         {:?}",
        parse_errors
            .iter()
            .map(|e| e.message.to_string())
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_unclosed_brace_message_propagation() {
    let engine = LintEngine::new(Config::default());
    let code = "def foo [] {";
    let violations = engine.lint_source(code, None);

    let parse_errors: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id.as_ref() == "nu_parse_error")
        .collect();

    assert!(!parse_errors.is_empty(), "Expected parse error");

    // Should describe the unclosed brace
    let has_meaningful_message = parse_errors.iter().any(|error| {
        let msg = error.message.to_string().to_lowercase();
        msg.contains("unclosed")
            || msg.contains("brace")
            || msg.contains("expected")
            || msg.contains("end of code")
            || msg.contains('}')
    });

    assert!(
        has_meaningful_message,
        "Parse error should describe the unclosed brace. Got: {:?}",
        parse_errors
            .iter()
            .map(|e| e.message.to_string())
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_unclosed_bracket_message_propagation() {
    let engine = LintEngine::new(Config::default());
    let code = "let x = [1, 2, 3";
    let violations = engine.lint_source(code, None);

    let parse_errors: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id.as_ref() == "nu_parse_error")
        .collect();

    assert!(!parse_errors.is_empty(), "Expected parse error");

    // Should describe the unclosed bracket
    let has_meaningful_message = parse_errors.iter().any(|error| {
        let msg = error.message.to_string().to_lowercase();
        msg.contains("unclosed")
            || msg.contains("bracket")
            || msg.contains("expected")
            || msg.contains("end of code")
            || msg.contains(']')
    });

    assert!(
        has_meaningful_message,
        "Parse error should describe the unclosed bracket. Got: {:?}",
        parse_errors
            .iter()
            .map(|e| e.message.to_string())
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_unexpected_token_message_propagation() {
    let engine = LintEngine::new(Config::default());
    let code = "let let x = 5";
    let violations = engine.lint_source(code, None);

    let parse_errors: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id.as_ref() == "nu_parse_error")
        .collect();

    assert!(!parse_errors.is_empty(), "Expected parse error");

    // All error messages should be descriptive
    for error in &parse_errors {
        let msg = error.message.to_string();
        assert!(!msg.is_empty(), "Error message should not be empty");
        assert!(msg.len() > 5, "Error message should be descriptive: {msg}");
    }
}

#[test]
fn test_invalid_function_definition_message_propagation() {
    let engine = LintEngine::new(Config::default());
    let code = "def [] { }";
    let violations = engine.lint_source(code, None);

    let parse_errors: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id.as_ref() == "nu_parse_error")
        .collect();

    assert!(!parse_errors.is_empty(), "Expected parse error");

    // Error messages should mention what's expected
    let has_meaningful_message = parse_errors.iter().any(|error| {
        let msg = error.message.to_string().to_lowercase();
        msg.contains("expected")
            || msg.contains("name")
            || msg.contains("identifier")
            || msg.contains("def")
    });

    assert!(
        has_meaningful_message,
        "Parse error should describe what's missing/expected. Got: {:?}",
        parse_errors
            .iter()
            .map(|e| e.message.to_string())
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_parse_error_message_not_empty() {
    let engine = LintEngine::new(Config::default());
    let code = "let x = (";
    let violations = engine.lint_source(code, None);

    let parse_errors: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id.as_ref() == "nu_parse_error")
        .collect();

    assert!(!parse_errors.is_empty(), "Expected parse error");

    // All parse errors must have non-empty messages
    for error in parse_errors {
        assert!(
            !error.message.is_empty(),
            "Parse error message should not be empty"
        );
    }
}

#[test]
fn test_parse_error_has_error_severity() {
    let engine = LintEngine::new(Config::default());
    let code = "let x = (";
    let violations = engine.lint_source(code, None);

    let parse_errors: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id.as_ref() == "nu_parse_error")
        .collect();

    assert!(!parse_errors.is_empty(), "Expected parse error");

    // All parse errors should have Error severity
    for error in parse_errors {
        use crate::violation::Severity;
        assert_eq!(
            error.severity,
            Severity::Error,
            "Parse errors should have Error severity"
        );
    }
}

#[test]
fn test_multiple_parse_errors_each_have_messages() {
    let engine = LintEngine::new(Config::default());
    let code = "let x = (\nlet y = [";
    let violations = engine.lint_source(code, None);

    let parse_errors: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id.as_ref() == "nu_parse_error")
        .collect();

    assert!(!parse_errors.is_empty(), "Expected parse errors");

    // Each error should have a meaningful message
    for error in parse_errors {
        assert!(
            !error.message.is_empty(),
            "Each error should have a message"
        );
        assert!(
            error.message.len() > 5,
            "Error message should be descriptive: {}",
            error.message
        );
    }
}
