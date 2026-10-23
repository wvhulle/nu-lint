use super::RULE;

#[test]
fn test_detect_simple_chained_get() {
    let bad_code = r#"
[[name value]; [foo 1] [bar 2]] | get name | get 0
"#;

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_triple_chained_get() {
    let bad_code = r#"
[[a]; [[1 2 3]]] | get a | get 0 | get 1
"#;

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_record_chained_get() {
    let bad_code = r#"
{foo: {bar: {baz: 42}}} | get foo | get bar
"#;

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_nested_list_access() {
    let bad_code = r#"
{data: [[1 2] [3 4]]} | get data | get 0 | get 1
"#;

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_string_and_int_mixed() {
    let bad_code = r#"
[[items]; [[{x: 1}]]] | get items | get 0
"#;

    RULE.assert_detects(bad_code);
}
