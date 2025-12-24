use super::RULE;

#[test]
fn detects_long_pipeline() {
    let code = r#"[1 2 3 4 5] | each { |x| $x * 2 } | where { |x| $x > 4 } | reduce { |it, acc| $acc + $it } | $in * 10 | into string"#;
    RULE.assert_detects(code);
}

#[test]
fn detects_long_pipeline_with_many_stages() {
    let code = r#"open data.csv | get column | str trim | str downcase | split row "," | flatten | uniq | sort | first 10"#;
    RULE.assert_detects(code);
}
