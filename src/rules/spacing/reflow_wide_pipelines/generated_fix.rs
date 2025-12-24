use super::RULE;

#[test]
fn test_fix_splits_pipeline_across_lines() {
    let bad_code = r#"[1 2 3 4 5] | each { |x| $x * 2 } | where { |x| $x > 4 } | reduce { |it, acc| $acc + $it } | $in * 10 | into string"#;
    RULE.assert_replacement_contains(bad_code, "\n| each");
    RULE.assert_replacement_contains(bad_code, "\n| where");
    RULE.assert_replacement_contains(bad_code, "\n| reduce");
}

#[test]
fn test_fix_preserves_first_element() {
    let bad_code = r#"open data.csv | get column | str trim | str downcase | split row "," | flatten | uniq | sort | first 10"#;
    RULE.assert_replacement_contains(bad_code, "open data.csv\n");
}

#[test]
fn test_fix_pipe_at_line_start() {
    let bad_code = r#"ls | where size > 1kb | get name | each { |f| open $f } | flatten | where type == "file" | sort | length"#;
    RULE.assert_replacement_contains(bad_code, "ls\n| where");
    RULE.assert_replacement_contains(bad_code, "\n| get name");
    RULE.assert_replacement_contains(bad_code, "\n| length");
}
