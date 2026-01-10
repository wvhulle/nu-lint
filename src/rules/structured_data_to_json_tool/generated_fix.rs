use super::RULE;

#[test]
fn fix_adds_to_json() {
    RULE.assert_fixed_is("ls | ^jq '.name'", "ls | to json | ^jq '.name'");
    RULE.assert_fixed_is("{ a: 1 } | ^jq '.a'", "{ a: 1 } | to json | ^jq '.a'");
    RULE.assert_fixed_is("[1, 2] | ^json_pp", "[1, 2] | to json | ^json_pp");
}
