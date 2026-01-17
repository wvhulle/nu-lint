use super::RULE;

#[test]
fn test_ignore_already_using_regex_operator() {
    let good_code = r#"
let result = $string =~ 'pattern'
"#;

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_negated_regex_operator() {
    let good_code = r#"
if $name !~ 'test' {
    print "no match"
}
"#;

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_str_contains_with_regex_special_chars() {
    let good_code = r#"
let result = ($text | str contains '*.txt')
"#;

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_str_contains_with_brackets() {
    let good_code = r#"
let found = ($str | str contains '[test]')
"#;

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_str_contains_with_parentheses() {
    let good_code = r#"
let has_group = ($data | str contains '(group)')
"#;

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_str_contains_with_dot() {
    let good_code = r#"
let has_dot = ($filename | str contains 'test.txt')
"#;

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_in_operator() {
    let good_code = r#"
let items = ['a', 'b', 'c']
let has_a = 'a' in $items
"#;

    RULE.assert_ignores(good_code);
}
