use super::rule;
use crate::LintContext;

#[test]
fn test_pipe_spacing() {
    let bad = "ls|get name";

    LintContext::test_with_parsed_source(bad, |context| {
        assert!(!(rule().check)(&context).is_empty());
    });
}

#[test]
fn test_closure_pipe_not_flagged_but_operators_are() {
    // But actual pipe operators should still be flagged
    let bad_with_closure = "{|x| echo $x}|get name";

    rule().assert_detects(bad_with_closure);
}

#[test]
fn test_detect_double_space_before_pipe() {
    let bad_code = "ls  | get name";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_double_space_before_pipe_no_space_after() {
    let bad_code = "ls  |get name";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_missing_space_after_pipe() {
    let bad_code = "ls| get name";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_double_space_after_pipe() {
    let bad_code = "ls |  get name";

    rule().assert_detects(bad_code);
}
