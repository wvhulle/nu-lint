use super::RULE;

#[test]
fn test_fix_str_contains_to_regex_operator() {
    let bad_code = "let result = ($text | str contains 'hello')";
    RULE.assert_fixed_contains(bad_code, "$text =~ 'hello'");
}

#[test]
fn test_fix_str_contains_simple() {
    let bad_code = "let result = ($some_string | str contains 'a')";
    RULE.assert_fixed_contains(bad_code, "$some_string =~ 'a'");
}

#[test]
fn test_fix_negated_str_contains_to_not_match() {
    let bad_code = "if not ($name | str contains 'test') { print 'no test found' }";
    RULE.assert_fixed_contains(bad_code, "$name !~ 'test'");
}

#[test]
fn test_fix_str_contains_in_if_condition() {
    let bad_code = "if ($input | str contains 'pattern') { print 'found' }";
    RULE.assert_fixed_contains(bad_code, "$input =~ 'pattern'");
}

#[test]
fn test_fix_negated_str_contains() {
    let bad_code = "let valid = not ($data | str contains 'error')";
    RULE.assert_fixed_contains(bad_code, "$data !~ 'error'");
}

#[test]
fn test_fix_explanation() {
    let bad_code = "($str | str contains 'x')";
    RULE.assert_fix_explanation_contains(bad_code, "=~");
}
