use super::RULE;

#[test]
fn ignore_to_json_before_jq() {
    RULE.assert_ignores("ls | to json | ^jq '.'");
}

#[test]
fn ignore_string_to_jq() {
    RULE.assert_ignores("'{\"a\": 1}' | ^jq '.a'");
}

#[test]
fn ignore_non_json_tool() {
    RULE.assert_ignores("ls | ^grep test");
}
