use super::RULE;

#[test]
fn ignore_last_command() {
    RULE.assert_ignores("[1, 2, 3] | last 2");
}

#[test]
fn ignore_slice_positive_start() {
    RULE.assert_ignores("[1, 2, 3] | slice 1..");
}

#[test]
fn ignore_slice_with_end() {
    RULE.assert_ignores("[1, 2, 3] | slice (-2)..-1");
}

#[test]
fn ignore_slice_zero_to_end() {
    RULE.assert_ignores("[1, 2, 3] | slice 0..");
}

#[test]
fn ignore_slice_with_step() {
    RULE.assert_ignores("[1, 2, 3, 4, 5] | slice (-2)..2..10");
}
