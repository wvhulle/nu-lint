use super::RULE;

#[test]
fn test_ignore_split_words_first() {
    let good = r#""hello world" | split words | first"#;
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_split_words_last() {
    let good = r#""hello world" | split words | last"#;
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_non_space_delimiter() {
    // Handled by split_row_first_last_to_parse_regex rule
    let good = r#""a:b:c" | split row ":" | first"#;
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_split_space_first_n() {
    // first 2 is different semantics - returns list
    let good = r#""hello world foo" | split row " " | first 2"#;
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_split_space_last_n() {
    // last 2 is different semantics - returns list
    let good = r#""hello world foo" | split row " " | last 2"#;
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_split_space_for_iteration() {
    let good = r#""hello world" | split row " " | each {|x| $x | str upcase }"#;
    RULE.assert_ignores(good);
}
