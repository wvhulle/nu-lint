use super::RULE;

#[test]
fn test_good_block_with_spaces() {
    let good = "do { print 'hello' }";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_if_block() {
    let good = "if true { echo 'yes' }";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_closure_without_params() {
    let good = "[1 2] | each { $in * 2 }";
    RULE.assert_ignores(good);
}

#[test]
fn test_closure_with_params_ignored() {
    // Closures WITH params are handled by closure_param_spacing rule
    let good = "[1 2] | each {|x| $x * 2 }";
    RULE.assert_ignores(good);
}

#[test]
fn test_record_ignored() {
    // Records are handled by record_brace_spacing rule
    let good = "{x: 1, y: 2}";
    RULE.assert_ignores(good);
}

#[test]
fn test_record_with_spaces_ignored() {
    // This rule doesn't check records (even bad ones)
    let record_with_spaces = "{ x: 1 }";
    RULE.assert_ignores(record_with_spaces);
}
