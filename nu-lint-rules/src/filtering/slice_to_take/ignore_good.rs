use super::RULE;

#[test]
fn ignore_take_command() {
    RULE.assert_ignores("[1, 2, 3] | take 2");
}

#[test]
fn ignore_slice_not_from_zero() {
    RULE.assert_ignores("[1, 2, 3] | slice 1..3");
}

#[test]
fn ignore_slice_open_ended() {
    RULE.assert_ignores("[1, 2, 3] | slice 0..");
}

#[test]
fn ignore_slice_negative_end() {
    RULE.assert_ignores("[1, 2, 3] | slice 0..-1");
}

#[test]
fn ignore_slice_with_step() {
    RULE.assert_ignores("[1, 2, 3, 4, 5] | slice 0..2..10");
}
