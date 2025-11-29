use super::rule;

#[test]
fn test_fix_simple_transpose_each() {
    let bad_code = r#"{a: [1, 2], b: [3]} | transpose key val | each {|row| $row.key }"#;
    rule().assert_replacement_contains(bad_code, "items {|key, val|");
    rule().assert_replacement_contains(bad_code, "$key");
}

#[test]
fn test_fix_with_both_fields() {
    let bad_code =
        r#"{x: [1], y: [2]} | transpose k v | each {|item| {key: $item.k, value: $item.v}}"#;
    rule().assert_replacement_contains(bad_code, "items {|k, v|");
    rule().assert_replacement_contains(bad_code, "$k");
    rule().assert_replacement_contains(bad_code, "$v");
}

#[test]
fn test_fix_group_by_pattern() {
    let bad_code = r#"open data.csv | group-by country | transpose country rows | each {|group| {country: $group.country, total: ($group.rows | length)}}"#;
    rule().assert_replacement_contains(bad_code, "items {|country, rows|");
    rule().assert_replacement_contains(bad_code, "$country");
    rule().assert_replacement_contains(bad_code, "$rows");
}

#[test]
fn test_fix_with_computation() {
    let bad_code =
        r#"{a: [1, 2, 3], b: [4, 5]} | transpose name values | each {|g| ($g.values | math sum)}"#;
    rule().assert_replacement_contains(bad_code, "items {|name, values|");
    rule().assert_replacement_contains(bad_code, "$values");
}

#[test]
fn test_fix_explanation() {
    let bad_code = r#"{a: [1], b: [2]} | transpose k v | each {|row| $row.k}"#;
    rule().assert_fix_explanation_contains(bad_code, "items");
    rule().assert_fix_explanation_contains(bad_code, "transpose");
}
