use super::RULE;

#[test]
fn test_ignore_split_row_without_first_last() {
    let good = r#""a:b:c" | split row ":""#;
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_split_row_with_get() {
    // Handled by split_row_index_to_parse rule
    let good = r#""a:b:c" | split row ":" | get 1"#;
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_split_row_first_n() {
    // first 2 is different semantics - returns list
    let good = r#""a:b:c" | split row ":" | first 2"#;
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_split_row_last_n() {
    // last 2 is different semantics - returns list
    let good = r#""a:b:c" | split row ":" | last 2"#;
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_parse_usage() {
    let good = r#""a:b:c" | parse "{first}:{rest}""#;
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_split_for_iteration() {
    let good = r#""a,b,c" | split row "," | each {|x| $x | str upcase }"#;
    RULE.assert_ignores(good);
}
