use super::RULE;

#[test]
fn test_good_closure_no_space_before_params() {
    // Style guide correct example
    let good = "[[status]; [UP] [UP]] | all {|el| $el.status == UP }";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_reduce_with_comma() {
    // Style guide correct example
    let good = "[1 2 3 4] | reduce {|elt, acc| $elt + $acc }";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_reduce_without_comma() {
    // Style guide correct example (no comma between params)
    let good = "[1 2 3 4] | reduce {|elt acc| $elt + $acc }";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_each_closure() {
    let good = "[1 2 3] | each {|x| $x * 2 }";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_closure_empty_params() {
    // Closure with empty params - no space before pipes
    let good = "[1 2] | each {|| $in * 2 }";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_closure_empty_params_in_list() {
    // User's original example - closure with empty params in list
    let good = "[{|| notify-long-command }]";
    RULE.assert_ignores(good);
}

#[test]
fn test_closure_without_params_ignored() {
    // Closures without explicit pipe delimiters are handled by block_body_spacing
    // rule
    let good = "do { print 'hello' }";
    RULE.assert_ignores(good);
}

#[test]
fn test_record_ignored() {
    // Records are handled by record_brace_spacing rule
    let good = "{x: 1, y: 2}";
    RULE.assert_ignores(good);
}
