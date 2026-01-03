use super::RULE;

#[test]
fn test_ignore_single_append() {
    let good_code = r#"$a | append $b"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_no_append() {
    let good_code = r#"$a | filter {|x| $x > 0}"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_already_spread_syntax() {
    let good_code = r#"[...$a, ...$b, ...$c]"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_spread_with_explicit_items() {
    let good_code = r#"[...$a, 1, 2, ...$b]"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_appends_with_large_gap() {
    let good_code = r#"$a | append $b | map {|x| $x * 2} | filter {|x| $x > 0} | append $c"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_different_operations() {
    let good_code = r#"$a | prepend $b | insert 0 $c"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_separate_single_appends() {
    let good_code = r#"
let a = $x | append $y
let b = $p | append $q
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_empty_list() {
    let good_code = r#"let result = []"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_prepend_operations() {
    let good_code = r#"$a | prepend $b | prepend $c"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_mixed_append_prepend() {
    let good_code = r#"$a | append $b | prepend $c"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_append_with_map_in_between() {
    let good_code = r#"$a | append $b | map {|x| $x * 2} | append $c"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_single_append_in_pipeline() {
    let good_code = r#"$data | filter {|x| $x.active} | append $new | sort"#;
    RULE.assert_ignores(good_code);
}
