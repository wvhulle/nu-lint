use super::RULE;

#[test]
fn test_fix_simple_transpose_each() {
    let bad_code = r#"{a: [1, 2], b: [3]} | transpose key val | each {|row| $row.key }"#;
    let expected = r#"{a: [1, 2], b: [3]} | items {|key, val| $key }"#;
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_with_both_fields() {
    let bad_code =
        r#"{x: [1], y: [2]} | transpose k v | each {|item| {key: $item.k, value: $item.v}}"#;
    let expected = r#"{x: [1], y: [2]} | items {|k, v| {key: $k, value: $v}}"#;
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_group_by_pattern() {
    let bad_code = r#"open data.csv | group-by country | transpose country rows | each {|group| {country: $group.country, total: ($group.rows | length)}}"#;
    let expected = r#"open data.csv | group-by country | items {|country, rows| {country: $country, total: ($rows | length)}}"#;
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_with_computation() {
    let bad_code =
        r#"{a: [1, 2, 3], b: [4, 5]} | transpose name values | each {|g| ($g.values | math sum)}"#;
    let expected = r#"{a: [1, 2, 3], b: [4, 5]} | items {|name, values| ($values | math sum)}"#;
    RULE.assert_fixed_is(bad_code, expected);
}
