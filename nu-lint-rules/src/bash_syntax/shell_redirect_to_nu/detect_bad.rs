use super::RULE;

#[test]
fn detect_merged_streams_before_pipe() {
    RULE.assert_detects("^make 2>&1 | ^grep error");
}

#[test]
fn detect_stderr_redirect_with_space() {
    RULE.assert_detects("^make 2> errors.txt");
}

#[test]
fn detect_stderr_redirect_without_space_that_parses_silently() {
    RULE.assert_detects("^make 2>/dev/null");
}

#[test]
fn detect_each_redirect_in_nested_block() {
    RULE.assert_count(
        "def build [] { ^make 2>&1 | ^grep a; ^make 2>&1 | ^grep b }",
        2,
    );
}

#[test]
fn detect_merged_streams_before_other_argument_without_fix() {
    RULE.assert_detects_without_fix("^make 2>&1 target");
}
