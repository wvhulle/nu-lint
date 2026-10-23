use super::RULE;

#[test]
fn detect_line_count() {
    RULE.assert_detects("$text | ^wc -l");
    RULE.assert_detects("$text | ^wc --lines");
    RULE.assert_detects("^wc -l notes.txt");
}
