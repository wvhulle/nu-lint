use super::rule;

#[test]
fn test_good_str_replace() {
    let good = "'hello world' | str replace 'world' 'universe'";
    rule().assert_ignores(good);
}

#[test]
fn test_good_str_replace_regex() {
    let good = "'email@domain.com' | str replace -r '\\w+@' 'user@'";
    rule().assert_ignores(good);
}

#[test]
fn test_good_str_replace_all() {
    let good = "'foo bar foo' | str replace -a 'foo' 'baz'";
    rule().assert_ignores(good);
}

#[test]
fn test_good_select_columns() {
    let good = "ls | select name size";
    rule().assert_ignores(good);
}

#[test]
fn test_good_select_with_range() {
    let good = "open data.csv | select 0..2";
    rule().assert_ignores(good);
}

#[test]
fn test_good_get_nested_field() {
    let good = "sys | get cpu.0.name";
    rule().assert_ignores(good);
}

#[test]
fn test_good_where_filter() {
    let good = "ls | where size > 1000";
    rule().assert_ignores(good);
}

#[test]
fn test_good_length_count() {
    let good = "ls | length";
    rule().assert_ignores(good);
}

#[test]
fn test_good_str_length() {
    let good = "'hello' | str length";
    rule().assert_ignores(good);
}

#[test]
fn test_good_tee_save() {
    let good = "ls | tee { save list.json }";
    rule().assert_ignores(good);
}

#[test]
fn test_good_tee_multiple_outputs() {
    let good = "ls | tee { save backup.json } { to yaml | save output.yaml }";
    rule().assert_ignores(good);
}

#[test]
fn test_good_str_trim_and_transform() {
    let good = "'  hello  ' | str trim | str upcase";
    rule().assert_ignores(good);
}

#[test]
fn test_good_str_case_conversion() {
    let good = "'HELLO' | str downcase";
    rule().assert_ignores(good);
}
