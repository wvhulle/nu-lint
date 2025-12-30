use super::RULE;

#[test]
fn test_fix_simple_transpose_each() {
    let bad_code = r#"{a: [1, 2], b: [3]} | transpose key val | each {|row| $row.key }"#;
    RULE.assert_fixed_contains(bad_code, "items");
    RULE.assert_fixed_contains(bad_code, "|key, val|");
    RULE.assert_fixed_contains(bad_code, "$key");
}

#[test]
fn test_fix_with_both_fields() {
    let bad_code =
        r#"{x: [1], y: [2]} | transpose k v | each {|item| {key: $item.k, value: $item.v}}"#;
    RULE.assert_fixed_contains(bad_code, "items");
    RULE.assert_fixed_contains(bad_code, "|k, v|");
    RULE.assert_fixed_contains(bad_code, "$k");
    RULE.assert_fixed_contains(bad_code, "$v");
}

#[test]
fn test_fix_group_by_pattern() {
    let bad_code = r#"open data.csv | group-by country | transpose country rows | each {|group| {country: $group.country, total: ($group.rows | length)}}"#;
    RULE.assert_fixed_contains(bad_code, "items");
    RULE.assert_fixed_contains(bad_code, "|country, rows|");
    RULE.assert_fixed_contains(bad_code, "$country");
    RULE.assert_fixed_contains(bad_code, "$rows");
}

#[test]
fn test_fix_with_computation() {
    let bad_code =
        r#"{a: [1, 2, 3], b: [4, 5]} | transpose name values | each {|g| ($g.values | math sum)}"#;
    RULE.assert_fixed_contains(bad_code, "items");
    RULE.assert_fixed_contains(bad_code, "|name, values|");
    RULE.assert_fixed_contains(bad_code, "$values");
}

#[test]
fn test_fix_explanation() {
    let bad_code = r#"{a: [1], b: [2]} | transpose k v | each {|row| $row.k}"#;
    RULE.assert_fix_explanation_contains(bad_code, "items");
    RULE.assert_fix_explanation_contains(bad_code, "transpose");
}
