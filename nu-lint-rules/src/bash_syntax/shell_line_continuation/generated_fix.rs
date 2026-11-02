use super::RULE;

#[test]
fn fix_joins_continued_line() {
    RULE.assert_fixed_is("^docker run \\\n    alpine", "^docker run alpine");
}

#[test]
fn fix_joins_crlf_continued_line() {
    RULE.assert_fixed_is("^docker run \\\r\n    alpine", "^docker run alpine");
}
