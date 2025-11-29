use super::rule;

#[test]
fn test_simple_transpose_each_pattern() {
    rule().assert_detects(
        r#"
        {a: [1, 2], b: [3]} | transpose key val | each {|row| $row.key }
        "#,
    );
}

#[test]
fn test_group_by_transpose_each() {
    rule().assert_detects(
        r#"
        open data.csv
        | group-by country
        | transpose country rows
        | each {|group| {country: $group.country, total: ($group.rows | length)}}
        "#,
    );
}

#[test]
fn test_transpose_each_both_fields() {
    rule().assert_detects(
        r#"
        {x: [1], y: [2]} | transpose k v | each {|item| {key: $item.k, value: $item.v}}
        "#,
    );
}

#[test]
fn test_transpose_each_with_computation() {
    rule().assert_detects(r#"
        {a: [1, 2, 3], b: [4, 5]} | transpose name values | each {|g| {name: $g.name, sum: ($g.values | math sum)}}
        "#);
}
