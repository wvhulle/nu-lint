use super::RULE;

#[test]
fn test_nothing_in_function_signature_return_type() {
    let good_code = r#"
def my-command []: nothing -> nothing {
    print "this returns nothing"
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_nothing_in_function_parameter_type() {
    let good_code = r#"
def my-command [x: nothing] {
    print "parameter with nothing type"
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_nothing_with_optional_parameter_default() {
    let good_code = r#"
def my-command [--opt: nothing = null] {
    print "optional parameter"
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_nothing_in_multiple_parameters() {
    let good_code = r#"
def my-command [x: nothing, y: string]: nothing -> nothing {
    print "multiple params with nothing"
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_null_in_function_body() {
    let good_code = r#"
def my-command [] {
    let x = null
    print $x
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_nothing_as_record_key() {
    let good_code = r#"
let dark_theme = { string: white, nothing: white, binary: white }
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_nothing_as_table_column() {
    let good_code = "[[nothing value]; [1 2]]";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_nothing_as_cell_path_argument() {
    let good_code = "{ nothing: 1 } | get nothing";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_quoted_nothing_string() {
    let good_code = r#"let s = "nothing""#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_nothing_as_string_typed_argument() {
    let good_code = r#""something" | str contains nothing"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_nothing_as_external_command_argument() {
    let good_code = "^grep nothing file.txt";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_external_command_named_nothing_with_arguments() {
    let good_code = "^nothing --flag";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_no_nothing_keyword() {
    let good_code = r#"
def my-command [] {
    let x = 5
    print $x
}
"#;
    RULE.assert_ignores(good_code);
}
