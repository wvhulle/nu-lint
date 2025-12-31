use super::RULE;

#[test]
fn test_good_brace_spacing_with_spaces() {
    // Blocks without parameters should have spaces
    let good = "do { print 'hello' }";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_brace_spacing_without_spaces() {
    // Records should not have spaces
    let good = "{x: 1, y: 2}";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_closure_without_space() {
    let good = "[[status]; [UP]] | all {|el| $el.status == UP }";
    RULE.assert_ignores(good);
}

#[test]
fn test_multiline_record_ignored() {
    let good = r"{
    key1: value1
    key2: value2
}";
    RULE.assert_ignores(good);
}

#[test]
fn test_multiline_record_with_spacing_ignored() {
    let good = r"{ 
    key1: value1
    key2: value2
 }";
    RULE.assert_ignores(good);
}

#[test]
fn test_record_assigned_to_variable() {
    // Record assigned to a variable - no spaces inside braces
    let good = "let page = {header: $header_data, document: $doc}";
    RULE.assert_ignores(good);
}

#[test]
fn test_record_with_variables() {
    // Record with variable values - no spaces
    let good = "let data = {name: $name, value: $value}";
    RULE.assert_ignores(good);
}
