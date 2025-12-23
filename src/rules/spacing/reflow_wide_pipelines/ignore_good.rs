use super::rule;

#[test]
fn ignores_short_pipeline() {
    let code = "ls | where size > 1kb";
    rule().assert_ignores(code);
}

#[test]
fn ignores_two_element_pipeline() {
    let code = r#"open file.txt | lines | each { |line| $line | str trim } | save output.txt"#;
    rule().assert_ignores(code);
}

#[test]
fn ignores_multiline_pipeline() {
    let code = r#"[1 2 3 4 5]
| each { |x| $x * 2 }
| where { |x| $x > 4 }
| reduce { |it, acc| $acc + $it }"#;
    rule().assert_ignores(code);
}

#[test]
fn ignores_pipeline_within_length_limit() {
    let code = "ls | where size > 1kb | get name | first 5";
    rule().assert_ignores(code);
}
