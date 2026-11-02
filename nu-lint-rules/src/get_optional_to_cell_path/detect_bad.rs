use super::RULE;

#[test]
fn detect_short_flag_on_nested_path() {
    RULE.assert_detects("let sample = {a: {b: 1}}\n$sample | get -o a.b");
}

#[test]
fn detect_long_flag() {
    RULE.assert_detects("let sample = {a: 1}\n$sample | get --optional a");
}

#[test]
fn detect_with_following_command() {
    RULE.assert_detects("let sample = {a: 1}\n$sample | get -o a | default 0");
}

#[test]
fn detect_inside_def() {
    RULE.assert_detects("def read-model [payload: record] { $payload | get -o model.name }");
}
