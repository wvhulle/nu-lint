use super::rule;

#[test]
fn test_items_already_used() {
    rule().assert_ignores(
        r#"
        {a: [1 2], b: [3]} | items {|key, val| {key: $key, value: $val}}
        "#,
    );
}

#[test]
fn test_transpose_without_each() {
    rule().assert_ignores(
        r#"
        {a: [1], b: [2]} | transpose key val
        "#,
    );
}

#[test]
fn test_transpose_with_wrong_number_of_args() {
    rule().assert_ignores(
        r#"
        [[x y]; [1 2]] | transpose | each {|row| $row}
        "#,
    );
}

#[test]
fn test_transpose_with_one_arg() {
    rule().assert_ignores(
        r#"
        {a: [1], b: [2]} | transpose key | each {|row| $row.key}
        "#,
    );
}

#[test]
fn test_transpose_with_three_args() {
    rule().assert_ignores(
        r#"
        {a: [1], b: [2]} | transpose x y z | each {|row| $row.x}
        "#,
    );
}

#[test]
fn test_each_uses_row_variable_directly() {
    rule().assert_ignores(
        r#"
        {a: [1], b: [2]} | transpose k v | each {|row| $row | get k}
        "#,
    );
}

#[test]
fn test_each_accesses_other_fields() {
    rule().assert_ignores(
        r#"
        {a: [1], b: [2]} | transpose k v | each {|row| $row.k + $row.other}
        "#,
    );
}

#[test]
fn test_each_with_multiple_parameters() {
    rule().assert_ignores(
        r#"
        {a: [1], b: [2]} | transpose k v | each {|row, idx| {key: $row.k, index: $idx}}
        "#,
    );
}

#[test]
fn test_each_without_closure() {
    rule().assert_ignores(
        r#"
        {a: [1], b: [2]} | transpose k v | each $some_var
        "#,
    );
}

#[test]
fn test_transpose_separated_from_each() {
    rule().assert_ignores(
        r#"
        let data = {a: [1], b: [2]} | transpose k v
        $data | each {|row| $row.k}
        "#,
    );
}

#[test]
fn test_each_closure_without_field_usage() {
    rule().assert_ignores(
        r#"
        {a: [1], b: [2]} | transpose k v | each {|row| 42}
        "#,
    );
}

#[test]
fn test_field_name_in_string_literal() {
    rule().assert_detects(
        r#"
        {a: [1], b: [2]} | transpose k v | each {|row| {msg: "row.k is useful", val: $row.k}}
        "#,
    );
}
