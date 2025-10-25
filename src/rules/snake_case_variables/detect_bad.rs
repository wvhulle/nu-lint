use super::rule;

#[test]
fn test_bad_let_variables() {
    let bad_code = r#"
def bad-func [] {
    let myVariable = 5
    let AnotherVariable = 10
    let CamelCase = "bad"
}
"#;

    rule().assert_violation_count(bad_code, 3);
    rule().assert_violation_count_exact(bad_code, 3);
}

#[test]
fn test_bad_mut_variables() {
    let bad_code = "
def bad-func [] {
    mut MyCounter = 0
    mut totalSum = 100
    $MyCounter += 1
}
";

    rule().assert_violation_count_exact(bad_code, 2);
}

#[test]
fn test_bad_shadowed_variables() {
    // Nushell allows variable shadowing - all shadowed variables should still use
    // snake_case
    let bad_code = "
def shadow-test [] {
    let myVar = 42
    let myVar = $myVar + 1
    let myVar = $myVar * 2
}
";

    rule().assert_violation_count(bad_code, 3);
}

#[test]
fn test_bad_variables_with_various_naming_patterns() {
    let bad_code = r#"
def test-func [] {
    let HTTPRequest = "bad"
    let xmlParser = "bad"
    let URLEncoder = "bad"
    let firstName = "John"
}
"#;

    rule().assert_violation_count_exact(bad_code, 4);
}
