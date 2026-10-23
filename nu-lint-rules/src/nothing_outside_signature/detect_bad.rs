use super::RULE;

#[test]
fn test_nothing_in_if_condition() {
    let bad_code = r#"
if $x == nothing {
    print "x is nothing"
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_nothing_in_variable_assignment() {
    let bad_code = "let x = nothing";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_nothing_in_function_body() {
    let bad_code = r#"
def my-command [] {
    nothing
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_nothing_in_record() {
    let bad_code = r#"
let record = { value: nothing }
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_nothing_as_record_key_and_value() {
    let bad_code = "let record = { nothing: nothing }";
    RULE.assert_count(bad_code, 1);
}

#[test]
fn test_nothing_in_table_cell() {
    let bad_code = "[[a]; [nothing]]";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_nothing_in_list() {
    let bad_code = r#"
let items = [1, 2, nothing, 4]
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_nothing_in_string_interpolation() {
    let bad_code = r#"
let msg = $"Value is: (nothing)"
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_multiple_nothing_usages() {
    let bad_code = r#"
let x = nothing
let y = nothing
"#;
    RULE.assert_count(bad_code, 2);
}

#[test]
fn test_nothing_in_pipeline() {
    let bad_code = r#"
nothing | print
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_nothing_as_function_argument() {
    let bad_code = r#"
def my-func [value] { $value }
my-func nothing
"#;
    RULE.assert_detects(bad_code);
}
