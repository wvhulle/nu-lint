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
