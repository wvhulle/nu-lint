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

#[test]
fn detect_camel_case_variable_with_type_annotation() {
    let bad_code = "let myVar: int = 1; print $myVar";

    RULE.assert_count(bad_code, 1);
}

#[test]
fn name_typed_variable_without_type_annotation_colon() {
    use nu_lint_analysis::context::LintContext;

    let bad_code = "let myVar: record<x: int, y: int> = {x: 1, y: 2}; print $myVar";
    let violations = LintContext::test_get_violations(bad_code, |context| RULE.check(context));

    let messages: Vec<_> = violations
        .iter()
        .map(|violation| violation.message.as_ref())
        .collect();
    assert_eq!(
        messages,
        ["Variable 'myVar' should use snake_case naming convention"]
    );
}
