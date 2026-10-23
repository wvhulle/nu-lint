use super::RULE;

#[test]
fn ignore_skip_command() {
    RULE.assert_ignores("[1, 2, 3] | skip 2");
}

#[test]
fn ignore_slice_with_end() {
    RULE.assert_ignores("[1, 2, 3] | slice 1..3");
}

#[test]
fn ignore_slice_from_start() {
    RULE.assert_ignores("[1, 2, 3] | slice 0..5");
}

#[test]
fn ignore_slice_negative_start() {
    RULE.assert_ignores("[1, 2, 3] | slice (-2)..");
}

#[test]
fn ignore_slice_zero_to_end() {
    RULE.assert_ignores("[1, 2, 3] | slice 0..");
}

#[test]
fn ignore_slice_with_step() {
    RULE.assert_ignores("[1, 2, 3, 4, 5] | slice 1..2..10");
}
