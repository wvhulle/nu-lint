use super::RULE;

#[test]
fn ignore_first_with_count() {
    // first N returns a list, doesn't panic on empty
    RULE.assert_ignores("$items | first 5");
}

#[test]
fn ignore_last_with_count() {
    // last N returns a list, doesn't panic on empty
    RULE.assert_ignores("$items | last 3");
}

#[test]
fn ignore_first_with_one() {
    RULE.assert_ignores("$items | first 1");
}

#[test]
fn ignore_last_with_one() {
    RULE.assert_ignores("$items | last 1");
}

#[test]
fn ignore_inside_try_block() {
    RULE.assert_ignores("try { $list | first }");
}

#[test]
fn ignore_first_with_variable_count() {
    RULE.assert_ignores("$items | first $count");
}
