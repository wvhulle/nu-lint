use super::RULE;

#[test]
fn fix_simple_case() {
    let code = r#"$record | get -o key | is-empty"#;
    RULE.assert_fixed_contains(code, "$record not-has key");
}

#[test]
fn fix_with_variable_key() {
    let code = r#"$record | get -o $key | is-empty"#;
    RULE.assert_fixed_contains(code, "$record not-has $key");
}

#[test]
fn fix_with_inline_record() {
    let code = r#"{a: 1, b: 2} | get -o c | is-empty"#;
    RULE.assert_fixed_contains(code, "{a: 1, b: 2} not-has c");
}

#[test]
fn fix_inside_where_closure() {
    let code = r#"$targets | where {|t| $available | get -i $t | is-empty }"#;
    RULE.assert_fixed_contains(code, "$available not-has $t");
    // Ensure no duplicate $available
    RULE.assert_fixed_not_contains(code, "$available | $available");
}

#[test]
fn fix_inside_each_closure() {
    let code = r#"$items | each {|it| $record | get -o $it | is-empty }"#;
    RULE.assert_fixed_contains(code, "$record not-has $it");
}

#[test]
fn fix_with_pipeline_before_get() {
    let code = r#"$data | select field | get -o key | is-empty"#;
    RULE.assert_fixed_contains(code, "$data | select field not-has key");
}
