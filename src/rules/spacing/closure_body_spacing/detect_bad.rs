use super::RULE;

#[test]
fn test_closure_missing_space_after_params() {
    let bad = "[1 2] | each {|x|$x * 2 }";
    RULE.assert_detects(bad);
}

#[test]
fn test_closure_missing_space_before_closing_brace() {
    let bad = "[1 2] | each {|x| $x * 2}";
    RULE.assert_detects(bad);
}

#[test]
fn test_closure_missing_both_spaces() {
    let bad = "[1 2] | each {|x|$x * 2}";
    RULE.assert_detects(bad);
}

#[test]
fn test_empty_params_missing_space_after() {
    let bad = "[{||notify-long-command }]";
    RULE.assert_detects(bad);
}

#[test]
fn test_empty_params_missing_space_before_closing() {
    let bad = "[{|| notify-long-command}]";
    RULE.assert_detects(bad);
}
