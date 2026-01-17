use super::RULE;

#[test]
fn test_ignore_split_row_without_first() {
    let good = r#""a:b:c" | split row ":""#;
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_split_row_with_get() {
    // Handled by split_row_get_inline rule
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
fn test_ignore_split_row_last() {
    // last has no simple parse equivalent - would need greedy/non-greedy
    let good = r#""a:b:c" | split row ":" | last"#;
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
fn test_ignore_split_row_for_iteration() {
    let good = r#""a,b,c" | split row "," | each {|x| $x | str upcase }"#;
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_split_words() {
    let good = r#""hello world" | split words | first"#;
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_space_delimiter() {
    // Handled by split_row_space_to_split_words rule
    let good = r#""hello world" | split row " " | first"#;
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_split_column() {
    // split column has different semantics (creates table)
    let good = r#""a:b:c" | split column ":" | first"#;
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_split_chars() {
    // split chars has no delimiter
    let good = r#""abc" | split chars | first"#;
    RULE.assert_ignores(good);
}
