use super::RULE;

#[test]
fn detect_string_split_by_wrapping() {
    RULE.assert_detects("^echo 'first half\n    second half'");
}

#[test]
fn detect_argument_split_onto_deeper_line() {
    RULE.assert_detects("^grep -a -c\n    pattern");
}

#[test]
fn detect_each_line_of_command_split_twice() {
    RULE.assert_count("^grep -a\n    -c\n    pattern", 2);
}

#[test]
fn detect_common_indentation() {
    RULE.assert_detects("    ^echo first\n    ^echo second");
}
