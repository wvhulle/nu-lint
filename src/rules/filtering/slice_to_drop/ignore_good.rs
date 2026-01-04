use super::RULE;

#[test]
fn ignore_drop_command() {
    RULE.assert_ignores("[1, 2, 3] | drop 2");
}

#[test]
fn ignore_slice_positive_end() {
    RULE.assert_ignores("[1, 2, 3] | slice 0..2");
}

#[test]
fn ignore_slice_open_ended() {
    RULE.assert_ignores("[1, 2, 3] | slice 1..");
}

#[test]
fn ignore_slice_non_zero_start_negative_end() {
    RULE.assert_ignores("[1, 2, 3] | slice 1..-1");
}

#[test]
fn ignore_slice_to_minus_1() {
    RULE.assert_ignores("[1, 2, 3] | slice ..-1");
}

#[test]
fn ignore_slice_0_to_minus_1() {
    RULE.assert_ignores("[1, 2, 3] | slice 0..-1");
}

#[test]
fn ignore_slice_with_step() {
    RULE.assert_ignores("[1, 2, 3, 4, 5] | slice 0..2..-1");
}
