use super::RULE;

#[test]
fn fix_token_broken_at_wrap_width_without_space() {
    RULE.assert_fixed_is(
        "^echo '{\"httpStatus\":20\n  0}' | ^cat",
        "^echo '{\"httpStatus\":200}' | ^cat",
    );
}

#[test]
fn fix_word_wrap_with_space() {
    RULE.assert_fixed_is(
        "^grep --count --ignore-case --line-number\n    pattern",
        "^grep --count --ignore-case --line-number pattern",
    );
}

#[test]
fn fix_shorter_string_line_with_space() {
    RULE.assert_fixed_is(
        "^echo 'short\n    a much longer second line' | ^cat",
        "^echo 'short a much longer second line' | ^cat",
    );
}

#[test]
fn fix_removes_common_indentation() {
    RULE.assert_fixed_is(
        "    ^echo first\n    ^echo second",
        "^echo first\n^echo second",
    );
}

#[test]
fn fix_longest_line_ending_in_short_word_with_space() {
    RULE.assert_fixed_is(
        "^echo 'a long sentence that wraps\n  here'",
        "^echo 'a long sentence that wraps here'",
    );
}
