use super::RULE;

#[test]
fn detect_filter_with_closure() {
    RULE.assert_detects("[1 2] | filter {|x| $x > 1}");
}

#[test]
fn detect_filter_with_stored_closure() {
    RULE.assert_detects("let positive = {|x| $x > 0}; [1 -2] | filter $positive");
}

#[test]
fn detect_filter_inside_function() {
    RULE.assert_detects("def keep-big [] { filter {|x| $x > 10} }");
}
