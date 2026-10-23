use super::RULE;

#[test]
fn detect_structured_data_to_json_tools() {
    for code in [
        "ls | ^jq '.'",
        "{ a: 1 } | ^jq '.a'",
        "[1, 2, 3] | ^jq '.[]'",
        "ls | ^json_pp",
        "{ name: 'test' } | ^jsonlint",
        "ls | ^yq '.[0]'",
    ] {
        RULE.assert_detects(code);
    }
}

#[test]
fn detect_tables_to_csv_tools() {
    for code in [
        "ls | ^csvcut -c name",
        "ls | ^csvstat",
        "ls | ^csvgrep -c name -m test",
        "ls | ^csvlook",
        "[{a: 1} {a: 2}] | ^xsv table",
    ] {
        RULE.assert_detects(code);
    }
}
