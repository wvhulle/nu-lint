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
fn test_no_nothing_keyword() {
    let good_code = r#"
def my-command [] {
    let x = 5
    print $x
}
"#;
    RULE.assert_ignores(good_code);
}
