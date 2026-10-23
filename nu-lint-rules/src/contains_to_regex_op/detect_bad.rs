use super::RULE;

#[test]
fn test_detect_str_contains_in_subexpression() {
    let bad_code = r#"
let result = ($text | str contains 'hello')
"#;

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_str_contains_simple() {
    let bad_code = "let result = ($some_string | str contains 'a')";

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_negated_str_contains() {
    let bad_code = r#"
if not ($name | str contains 'test') {
    print "no test found"
}
"#;

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_str_contains_in_if_condition() {
    let bad_code = r#"
if ($input | str contains 'pattern') {
    print "found"
}
"#;

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_negated_str_contains_assignment() {
    let bad_code = r#"
let valid = not ($data | str contains 'error')
"#;

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_str_contains_with_whitespace_pattern() {
    let bad_code = r#"let has_space = ($text | str contains ' test ')"#;
    RULE.assert_detects(bad_code);
}
