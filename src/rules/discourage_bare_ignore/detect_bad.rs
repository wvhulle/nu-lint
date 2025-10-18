
use super::rule;

#[test]
fn test_bare_ignore_detected() {
    let bad_code = r"
some | pipeline | each { |x| process $x } | ignore
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_bare_ignore_complex_pipeline() {
    let bad_code = "some | pipeline | each { |x| process $x } | ignore";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_bare_ignore_simple() {
    let bad_code = "another | operation | ignore";

    rule().assert_detects(bad_code);
}
