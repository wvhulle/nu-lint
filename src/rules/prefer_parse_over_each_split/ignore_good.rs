use super::rule;

#[test]
fn test_ignore_parse_usage() {
    let good_code = r#"
open data.txt | parse "{name} {value}"
"#;

    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_each_without_split() {
    let good_code = r"
seq 1 10 | each { |x| $x * 2 }
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_split_without_each() {
    let good_code = r#"
"one,two,three" | split row ","
"#;

    rule().assert_ignores(good_code);
}
