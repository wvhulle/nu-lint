use super::RULE;

#[test]
fn detect_filter_on_file() {
    for code in [
        "^jq '.name' user.json",
        "^jq '.user.email' data.json",
        "^jq '.[0]' items.json",
        "^jq '.[-1]' items.json",
        "^jq 'sort' numbers.json",
        "^jq -r '.name' user.json",
        r#"^jq '.["test.write"]' settings.json"#,
    ] {
        RULE.assert_detects(code);
    }
}

#[test]
fn detect_filter_on_serialized_value() {
    RULE.assert_detects("$data | to json | ^jq '.field'");
    RULE.assert_detects("$users | to json | ^jq 'map(.name)'");
}

#[test]
fn detect_filter_on_external_output() {
    RULE.assert_detects("^curl -s https://example.com/api | ^jq '.name'");
}

#[test]
fn detect_filter_on_string() {
    RULE.assert_detects(r#"let text = '{"a": 1}'; $text | ^jq '.a'"#);
}

#[test]
fn detect_interpolated_filters() {
    RULE.assert_detects(r#"let field = "a"; ^jq $".($field)" data.json"#);
    RULE.assert_detects(r#"let idx = 1; ^jq $".items[($idx)]" data.json"#);
}

#[test]
fn detect_in_function() {
    RULE.assert_detects("def name [file] { ^jq '.name' $file }");
}
