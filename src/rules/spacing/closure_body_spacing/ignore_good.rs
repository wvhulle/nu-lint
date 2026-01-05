use super::RULE;

#[test]
fn test_good_closure_with_params_proper_spacing() {
    let good = "[1 2] | each {|x| $x * 2 }";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_closure_empty_params_proper_spacing() {
    let good = "[{|| notify-long-command }]";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_reduce_closure() {
    let good = "[1 2 3 4] | reduce {|elt, acc| $elt + $acc }";
    RULE.assert_ignores(good);
}

#[test]
fn test_block_without_pipes_ignored() {
    // Blocks without pipes are handled by block_body_spacing
    let good = "do { print 'hello' }";
    RULE.assert_ignores(good);
}

#[test]
fn test_record_ignored() {
    let good = "{x: 1, y: 2}";
    RULE.assert_ignores(good);
}
