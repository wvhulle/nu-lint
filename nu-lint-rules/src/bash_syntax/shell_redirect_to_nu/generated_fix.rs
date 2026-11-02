use super::RULE;

#[test]
fn fix_merged_streams_before_pipe() {
    RULE.assert_fixed_is("^make 2>&1 | ^grep error", "^make out+err>| ^grep error");
}

#[test]
fn fix_removes_trailing_merged_streams() {
    RULE.assert_fixed_is("^make 2>&1", "^make");
}

#[test]
fn fix_stderr_redirect_with_space() {
    RULE.assert_fixed_is("^make 2> errors.txt", "^make err> errors.txt");
}

#[test]
fn fix_stderr_redirect_without_space() {
    RULE.assert_fixed_is("^make 2>/dev/null", "^make err> /dev/null");
}
