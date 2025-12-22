use super::rule;

#[test]
fn ignore_with_short_optional_flag() {
    rule().assert_ignores("$record | get -o $key");
}

#[test]
fn ignore_with_long_optional_flag() {
    rule().assert_ignores("$record | get --optional $key");
}

#[test]
fn ignore_static_string_key() {
    rule().assert_ignores("$record | get name");
}

#[test]
fn ignore_quoted_static_key() {
    rule().assert_ignores(r#"$record | get "static_key""#);
}

#[test]
fn ignore_raw_string_key() {
    rule().assert_ignores(r#"$record | get r#'key'#"#);
}

#[test]
fn ignore_integer_key() {
    rule().assert_ignores("$list | get 0");
}

#[test]
fn ignore_cell_path_literal() {
    rule().assert_ignores("$record | get field.nested");
}

#[test]
fn ignore_optional_with_cell_path_key() {
    rule().assert_ignores("$data | get -o $item.field");
}

#[test]
fn ignore_get_without_arguments() {
    rule().assert_ignores("$record | get");
}

#[test]
fn ignore_default_after_optional_get() {
    rule().assert_ignores("$record | get -o $key | default 'fallback'");
}
