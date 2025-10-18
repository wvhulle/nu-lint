use super::rule;

#[test]
fn test_bad_completion_naming_detected() {
    let bad_code = r"def complete-branches [] { ^git branch }";

    rule().assert_detects(bad_code);
}
