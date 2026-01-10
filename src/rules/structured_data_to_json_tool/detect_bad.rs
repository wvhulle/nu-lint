use super::RULE;

#[test]
fn detect_structured_data_to_jq() {
    RULE.assert_detects("ls | ^jq '.'");
    RULE.assert_detects("{ a: 1 } | ^jq '.a'");
    RULE.assert_detects("[1, 2, 3] | ^jq '.[]'");
}

#[test]
fn detect_all_json_tools() {
    RULE.assert_detects("ls | ^json_pp");
    RULE.assert_detects("{ name: 'test' } | ^jsonlint");
}
