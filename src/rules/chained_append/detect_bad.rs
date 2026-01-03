use super::RULE;

#[test]
fn test_detect_simple_consecutive_appends() {
    let bad_code = r#"$a | append $b | append $c"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_three_append_chain() {
    let bad_code = r#"
let result = $x | append $y | append $z
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_four_append_chain() {
    let bad_code = r#"$a | append $b | append $c | append $d"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_append_cluster_with_filter() {
    let bad_code = r#"$a | append $b | filter {|x| $x > 0} | append $c"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_append_cluster_with_where() {
    let bad_code = r#"$list | append $more | where $it.id > 0 | append $extra"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_append_cluster_with_sort() {
    let bad_code = r#"$a | append $b | sort | append $c"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_function_call_pattern() {
    let bad_code = r#"
let backends = $ddcci | append $kde | append $backlight
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_with_inline_list() {
    let bad_code = r#"$var | append [1 2] | append $other"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_with_subexpression() {
    let bad_code = r#"$var | append (get-data) | append $other"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_mixed_expressions() {
    let bad_code = r#"$a | append [1 2 3] | append (func-call) | append $d"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_count_single_violation_per_cluster() {
    let bad_code = r#"$a | append $b | append $c | append $d"#;
    RULE.assert_count(bad_code, 1);
}

#[test]
fn test_detect_multiple_separate_clusters() {
    let bad_code = r#"
let x = $a | append $b | append $c
let y = $d | append $e | append $f
"#;
    RULE.assert_count(bad_code, 2);
}
