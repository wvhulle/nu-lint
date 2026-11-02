use super::RULE;

#[test]
fn test_fix_simple_chained_get() {
    let bad_code = r#"
[[name value]; [foo 1] [bar 2]] | get name | get 0
"#;
    RULE.assert_fixed_contains(bad_code, "get name.0");
}

#[test]
fn test_fix_triple_chained_get() {
    let bad_code = r#"
[[a]; [[1 2 3]]] | get a | get 0 | get 1
"#;
    RULE.assert_fixed_contains(bad_code, "get a.0.1");
}

#[test]
fn test_fix_record_chained_get() {
    let bad_code = r#"
{foo: {bar: {baz: 42}}} | get foo | get bar
"#;
    RULE.assert_fixed_contains(bad_code, "get foo.bar");
}

#[test]
fn test_fix_nested_list_access() {
    let bad_code = r#"
{data: [[1 2] [3 4]]} | get data | get 0 | get 1
"#;
    RULE.assert_fixed_contains(bad_code, "get data.0.1");
}

#[test]
fn test_fix_count_single_violation() {
    let bad_code = r#"
{a: {b: 1}} | get a | get b
"#;
    RULE.assert_count(bad_code, 1);
}

#[test]
fn fix_keeps_optional_flags_as_optional_members() {
    RULE.assert_fixed_is(
        "{a: {b: 1}} | get -o a | get -o b",
        "{a: {b: 1}} | get a?.b?",
    );
}

#[test]
fn fix_keeps_optional_long_flags_as_optional_members() {
    RULE.assert_fixed_is(
        "{a: {b: 1}} | get --optional a | get --optional b",
        "{a: {b: 1}} | get a?.b?",
    );
}

#[test]
fn fix_writes_optional_member_suffix() {
    RULE.assert_fixed_is("{a: {b: 1}} | get a? | get b", "{a: {b: 1}} | get a?.b");
}
