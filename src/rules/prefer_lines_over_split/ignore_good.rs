use super::rule;

#[test]
fn test_ignore_lines_usage() {
    let good_code = r"
open file.txt | lines
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_split_row_with_other_delimiter() {
    let good_code = r#"
"a,b,c" | split row ","
"#;

    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_split_row_with_colon() {
    let good_code = r#"
"PATH=/usr/bin:/bin" | split row ":"
"#;

    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_split_row_with_space() {
    let good_code = r#"
"one two three" | split row " "
"#;

    rule().assert_ignores(good_code);
}
