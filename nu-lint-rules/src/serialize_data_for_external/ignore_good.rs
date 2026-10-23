use super::RULE;

#[test]
fn ignore_serialized_data() {
    RULE.assert_ignores("ls | to json | ^jq '.'");
    RULE.assert_ignores("ls | to csv | ^csvcut -c name");
}

#[test]
fn ignore_text_input() {
    RULE.assert_ignores(r#"'{"a": 1}' | ^jq '.a'"#);
    RULE.assert_ignores("'name,age\nAlice,30' | ^csvcut -c name");
}

#[test]
fn ignore_other_tools() {
    RULE.assert_ignores("ls | ^grep test");
}

#[test]
fn ignore_values_csv_cannot_represent() {
    RULE.assert_ignores("[1, 2, 3] | ^csvcut");
    RULE.assert_ignores("{ name: 'test' } | ^csvcut");
}
