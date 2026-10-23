use nu_lint_analysis::log::init_test_log;

use super::RULE;

#[test]
fn test_detect_lines_each_parse() {
    init_test_log();
    let bad_code = r#"$text | lines | each {|l| $l | parse "{key}:{value}" }"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_lines_each_parse_regex() {
    let bad_code = r#"$text | lines | each {|line| $line | parse --regex "(?P<k>.*):(?P<v>.*)" }"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_lines_each_parse_simple() {
    let bad_code = r#"$input | lines | each {|x| $x | parse "{a} {b}" }"#;
    RULE.assert_detects(bad_code);
}
