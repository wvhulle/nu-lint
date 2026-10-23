use super::RULE;

#[test]
fn detect_camel_case_let_variables() {
    let bad_code = r#"
def bad-func [] {
    let myVariable = 5
    let AnotherVariable = 10
    let CamelCase = "bad"
}
"#;

    RULE.assert_count(bad_code, 3);
}

#[test]
fn detect_camel_case_mut_variables() {
    let bad_code = "
def bad-func [] {
    mut MyCounter = 0
    mut totalSum = 100
    $MyCounter += 1
}
";

    RULE.assert_count(bad_code, 2);
}

#[test]
fn detect_camel_case_in_shadowed_variables() {
    // Nushell allows variable shadowing - all shadowed variables should still use
    // snake_case
    let bad_code = "
def shadow-test [] {
    let myVar = 42
    let myVar = $myVar + 1
    let myVar = $myVar * 2
}
";

    RULE.assert_count(bad_code, 3);
}

#[test]
fn detect_mixed_naming_convention_violations() {
    let bad_code = r#"
def test-func [] {
    let HTTPRequest = "bad"
    let xmlParser = "bad"
    let URLEncoder = "bad"
    let firstName = "John"
}
"#;

    RULE.assert_count(bad_code, 4);
}
