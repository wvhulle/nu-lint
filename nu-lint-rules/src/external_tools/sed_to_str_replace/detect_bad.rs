use super::RULE;

#[test]
fn detect_global_literal_substitution() {
    RULE.assert_detects("$text | ^sed 's/foo/bar/g'");
    RULE.assert_detects("$text | ^sed 's|/usr|/opt|g'");
    RULE.assert_detects("^sed 's/a+b/c/g' notes.txt");
}
