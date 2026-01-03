use super::RULE;

#[test]
fn test_ignore_split_with_each() {
    let good_code = r#"
let split = ("a,b,c" | split row ",")
$split | each {|item| print $item}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_already_using_parse() {
    let good_code = r#"
let parsed = ("a:b:c" | parse "{a}:{b}:{c}")
$parsed | get a
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_split_without_index_access() {
    let good_code = r#"
let split = ("a,b,c" | split row ",")
$split | length
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_simple_split_row_in_single_pipeline() {
    let good_code = r#"
"a,b,c" | split row ","
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_split_variable_never_used() {
    let good_code = r#"
let split = ("a:b:c" | split row ":")
42
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_variable_reassigned_to_non_split() {
    let good_code = r#"
let split = ("a:b:c" | split row ":")
let split = [1, 2, 3]
$split | get 1
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_field_access_not_numeric() {
    let good_code = r#"
let data = {name: "test", value: 42}
$data | get name
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_split_with_map() {
    let good_code = r#"
let split = ("a,b,c" | split row ",")
$split | each {|x| $x | str upcase}
"#;
    RULE.assert_ignores(good_code);
}
