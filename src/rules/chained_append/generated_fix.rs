use super::RULE;

#[test]
fn test_fix_simple_variables_uses_spread() {
    let bad_code = r#"$a | append $b | append $c"#;
    RULE.assert_fixed_contains(bad_code, "[...$a, ...$b, ...$c]");
}

#[test]
fn test_fix_three_elements() {
    let bad_code = r#"$x | append $y | append $z"#;
    RULE.assert_fixed_is(bad_code, "[...$x, ...$y, ...$z]");
}

#[test]
fn test_fix_four_elements() {
    let bad_code = r#"$a | append $b | append $c | append $d"#;
    RULE.assert_fixed_is(bad_code, "[...$a, ...$b, ...$c, ...$d]");
}

#[test]
fn test_fix_preserves_inline_list() {
    let bad_code = r#"$a | append [1 2] | append $c"#;
    RULE.assert_fixed_contains(bad_code, "...[1 2]");
}

#[test]
fn test_fix_preserves_subexpression() {
    let bad_code = r#"$a | append (get-data) | append $c"#;
    RULE.assert_fixed_contains(bad_code, "...(get-data)");
}

#[test]
fn test_fix_mixed_expressions() {
    let bad_code = r#"$a | append [1 2] | append (func) | append $d"#;
    let fixed = bad_code;
    RULE.assert_fixed_contains(fixed, "...$a");
    RULE.assert_fixed_contains(fixed, "...[1 2]");
    RULE.assert_fixed_contains(fixed, "...(func)");
    RULE.assert_fixed_contains(fixed, "...$d");
}

#[test]
fn test_fix_explanation_mentions_spread() {
    let bad_code = r#"$x | append $y | append $z"#;
    RULE.assert_fix_explanation_contains(bad_code, "spread");
}

#[test]
fn test_fix_in_assignment() {
    let bad_code = r#"
let result = $a | append $b | append $c
"#;
    RULE.assert_fixed_contains(bad_code, "[...$a, ...$b, ...$c]");
}

#[test]
fn test_fix_cluster_with_filter() {
    let bad_code = r#"$a | append $b | filter {|x| $x > 0} | append $c"#;
    RULE.assert_fixed_contains(bad_code, "...$a");
    RULE.assert_fixed_contains(bad_code, "...$b");
    RULE.assert_fixed_contains(bad_code, "...$c");
}

#[test]
fn test_help_contains_example() {
    let bad_code = r#"$a | append $b | append $c"#;
    RULE.assert_help_contains(bad_code, "spread syntax");
}

#[test]
fn test_labels_mention_append() {
    let bad_code = r#"$a | append $b | append $c"#;
    RULE.assert_labels_contain(bad_code, "append");
}

#[test]
fn test_labels_show_starting_list() {
    let bad_code = r#"$a | append $b | append $c"#;
    RULE.assert_labels_contain(bad_code, "Starting list");
}

#[test]
fn test_labels_show_first_and_last_append() {
    let bad_code = r#"$a | append $b | append $c"#;
    RULE.assert_labels_contain(bad_code, "First append");
    RULE.assert_labels_contain(bad_code, "Last append");
}
