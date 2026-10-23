use super::RULE;

#[test]
fn test_space_before_closure_params() {
    // Style guide: "no space is allowed" before |el|
    let bad = "[[status]; [UP]] | all { |el| $el.status == UP }";
    RULE.assert_detects(bad);
}

#[test]
fn test_multiple_spaces_before_params() {
    let bad = "[1 2 3] | each {  |x| $x * 2 }";
    RULE.assert_detects(bad);
}

#[test]
fn test_space_before_multiple_params() {
    // Style guide example: reduce with params
    let bad = "[1 2 3 4] | reduce { |elt, acc| $elt + $acc }";
    RULE.assert_detects(bad);
}

#[test]
fn test_newline_before_params() {
    let bad = "[1 2] | each {\n|x| $x }";
    RULE.assert_detects(bad);
}
