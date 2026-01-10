use super::RULE;

#[test]
fn ignore_to_csv_before_csvcut() {
    RULE.assert_ignores("ls | to csv | ^csvcut -c name");
}

#[test]
fn ignore_string_to_csvcut() {
    RULE.assert_ignores("'name,age\nAlice,30' | ^csvcut -c name");
}

#[test]
fn ignore_non_csv_tool() {
    RULE.assert_ignores("ls | ^jq '.'");
}

#[test]
fn ignore_non_table_types() {
    RULE.assert_ignores("[1, 2, 3] | ^csvcut");
    RULE.assert_ignores("{ name: 'test' } | ^csvcut");
}
