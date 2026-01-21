use super::RULE;

#[test]
fn test_fix_removes_unused_let() {
    let code = r#"let unused = 5
print "hello""#;
    RULE.assert_fix_erases(code, "let unused = 5");
}

#[test]
fn test_fix_removes_unused_mut() {
    let code = r#"mut unused = 5
print "hello""#;
    RULE.assert_fix_erases(code, "mut unused = 5");
}

#[test]
fn test_fix_unused_at_start() {
    let code = r#"let unused = 5
let used = 10
print $used"#;
    let expected = r#"let used = 10
print $used"#;
    RULE.assert_fixed_is(code, expected);
}

#[test]
fn test_fix_unused_in_middle() {
    let code = r#"let a = 1
let unused = 2
let b = 3
print ($a + $b)"#;
    let expected = r#"let a = 1
let b = 3
print ($a + $b)"#;
    RULE.assert_fixed_is(code, expected);
}

#[test]
fn test_fix_one_of_multiple_unused() {
    // When multiple violations exist, each fix is applied separately
    // This test verifies one unused variable is removed per fix application
    let code = r#"let x = 1
let y = 2
print "done""#;
    // After one fix, either x or y is removed (order is non-deterministic)
    let fixed = RULE.apply_first_fix(code);
    let x_removed = !fixed.contains("let x = 1");
    let y_removed = !fixed.contains("let y = 2");
    assert!(
        x_removed || y_removed,
        "Expected at least one of 'let x = 1' or 'let y = 2' to be removed, but got: {fixed}"
    );
}

#[test]
fn test_fix_preserves_used_variables() {
    let code = r#"let used = "hello"
let unused = "world"
print $used"#;
    let expected = r#"let used = "hello"
print $used"#;
    RULE.assert_fixed_is(code, expected);
}

#[test]
fn test_fix_unused_with_complex_rhs() {
    let code = r#"let unused = (1 + 2 * 3)
print "result""#;
    let expected = r#"print "result""#;
    RULE.assert_fixed_is(code, expected);
}

#[test]
fn test_fix_unused_mut_with_list() {
    let code = r#"mut unused = [1, 2, 3]
print "done""#;
    let expected = r#"print "done""#;
    RULE.assert_fixed_is(code, expected);
}
