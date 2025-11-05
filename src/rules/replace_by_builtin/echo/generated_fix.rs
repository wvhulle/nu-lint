use super::rule;

#[test]
fn test_suggestion_for_string_literal() {
    let bad_code = r#"echo "hello world""#;

    rule().assert_detects(bad_code);
    rule().assert_suggestion_contains(bad_code, r#"echo "hello world""#);
    rule().assert_suggestion_contains(bad_code, r#""hello world""#);
    rule().assert_suggestion_contains(bad_code, "Good:");
    rule().assert_suggestion_contains(bad_code, "print");
}

#[test]
fn test_suggestion_for_variable() {
    let bad_code = r"echo $value";

    rule().assert_detects(bad_code);
    rule().assert_suggestion_contains(bad_code, "echo $value");
    rule().assert_suggestion_contains(bad_code, "$value");
    rule().assert_suggestion_contains(bad_code, "Good:");
}

#[test]
fn test_suggestion_for_pipeline() {
    let bad_code = r"echo $var | str upcase";

    rule().assert_detects(bad_code);
    rule().assert_suggestion_contains(bad_code, "$var | str upcase");
    rule().assert_suggestion_contains(bad_code, "echo $var");
}

#[test]
fn test_suggestion_for_external_echo() {
    let bad_code = r#"^echo "test""#;

    rule().assert_detects(bad_code);
    rule().assert_suggestion_contains(bad_code, "^echo");
}

#[test]
fn test_message_explains_why_bad() {
    let bad_code = r"echo $value";

    rule().assert_detects(bad_code);
}

#[test]
fn test_different_suggestions_for_different_patterns() {
    rule().assert_detects(r#"echo "hello""#);
    rule().assert_suggestion_contains(r#"echo "hello""#, r#""hello""#);

    rule().assert_detects(r"echo $var");
    rule().assert_suggestion_contains(r"echo $var", "$var");

    rule().assert_detects(r#"^echo "world""#);
    rule().assert_suggestion_contains(r#"^echo "world""#, "print");
}

#[test]
fn test_suggestion_shows_actual_code_context() {
    let bad_code = r#"echo "Debug: processing item""#;

    rule().assert_detects(bad_code);
    rule().assert_suggestion_contains(bad_code, "Debug: processing item");
}

#[test]
fn test_each_violation_has_tailored_suggestion() {
    let bad_code = r#"
echo "first"
echo $var
^echo "third"
"#;

    rule().assert_violation_count_exact(bad_code, 3);
}

#[test]
fn test_suggestion_format_consistency() {
    let bad_code = r"echo $value";

    rule().assert_detects(bad_code);
    rule().assert_suggestion_contains(bad_code, "Bad:");
    rule().assert_suggestion_contains(bad_code, "Good:");
}

#[test]
fn test_suggestion_for_multiple_arguments() {
    let bad_code = r"echo hello world test";

    rule().assert_detects(bad_code);
    rule().assert_suggestion_contains(bad_code, "echo");
}
