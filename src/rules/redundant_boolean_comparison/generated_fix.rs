use super::RULE;

#[test]
fn test_fix_equal_true() {
    let bad_code = "if $flag == true { x }";
    let expected = "if $flag { x }";
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_equal_false() {
    let bad_code = "if $flag == false { x }";
    let expected = "if (not $flag) { x }";
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_not_equal_true() {
    let bad_code = "if $enabled != true { x }";
    let expected = "if (not $enabled) { x }";
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_not_equal_false() {
    let bad_code = "if $active != false { x }";
    let expected = "if $active { x }";
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_true_on_left_equal() {
    let bad_code = "if true == $flag { x }";
    let expected = "if $flag { x }";
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_false_on_left_equal() {
    let bad_code = "if false == $flag { x }";
    let expected = "if (not $flag) { x }";
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_true_on_left_not_equal() {
    let bad_code = "if true != $flag { x }";
    let expected = "if (not $flag) { x }";
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_false_on_left_not_equal() {
    let bad_code = "if false != $flag { x }";
    let expected = "if $flag { x }";
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_let_binding() {
    let bad_code = "let result = $check == true";
    let expected = "let result = $check";
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_explanation() {
    let bad_code = "if $flag == true { x }";
    RULE.assert_fix_explanation_contains(bad_code, "Simplify to:");
}
