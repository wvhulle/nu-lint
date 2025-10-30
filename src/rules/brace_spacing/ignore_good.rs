use super::rule;

#[test]
fn test_good_brace_spacing_with_spaces() {
    // Blocks without parameters should have spaces
    let good = "do { print 'hello' }";
    rule().assert_ignores(good);
}

#[test]
fn test_good_brace_spacing_without_spaces() {
    // Records should not have spaces
    let good = "{x: 1, y: 2}";
    rule().assert_ignores(good);
}

#[test]
fn test_good_closure_without_space() {
    let good = "[[status]; [UP]] | all {|el| $el.status == UP }";
    rule().assert_ignores(good);
}
