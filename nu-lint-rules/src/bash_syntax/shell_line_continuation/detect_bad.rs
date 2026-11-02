use super::RULE;

#[test]
fn detect_continuation_on_every_line() {
    RULE.assert_count("^docker run \\\n  --rm \\\n  alpine", 2);
}

#[test]
fn detect_continuation_with_crlf_line_ending() {
    RULE.assert_detects("^docker run \\\r\n  alpine");
}

#[test]
fn detect_continuation_in_nested_block() {
    RULE.assert_detects("def start [] {\n  ^docker run \\\n    alpine\n}");
}
