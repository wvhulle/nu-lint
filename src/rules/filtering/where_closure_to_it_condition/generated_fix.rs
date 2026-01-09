use super::RULE;

#[test]
fn fix_simple_closure() {
    let source = r"[1, 2, 3] | where {|x| $x > 2}";
    let expected = r"[1, 2, 3] | where { $it > 2}";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_is(source, expected);
}

#[test]
fn fix_closure_with_different_param() {
    let source = r"[1, 2, 3] | where {|num| $num > 2}";
    let expected = r"[1, 2, 3] | where { $it > 2}";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_is(source, expected);
}

#[test]
fn fix_closure_with_line_param() {
    let source = r#"open --raw file.txt | lines | where {|line| $line =~ "pattern"}"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"$it =~ "pattern""#);
}

#[test]
fn fix_closure_with_field_access() {
    let source = r"ls | where {|f| $f.size > 100kb}";
    let expected = r"ls | where { $it.size > 100kb}";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_is(source, expected);
}

#[test]
fn fix_closure_with_item_param() {
    let source = r#"open data.json | where {|item| $item.field == "value"}"#;
    let expected = r#"open data.json | where { $it.field == "value"}"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_is(source, expected);
}

#[test]
fn fix_closure_with_pipeline() {
    let source = r"[1, 2, 3] | where {|x| ($x | str length) > 0}";
    let expected = r"[1, 2, 3] | where { ($it | str length) > 0}";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_is(source, expected);
}

#[test]
fn fix_closure_with_external_variable() {
    let source = r"
let threshold = 2
[1, 2, 3] | where {|x| $x > $threshold}
";
    let expected = r"
let threshold = 2
[1, 2, 3] | where { $it > $threshold}
";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_is(source, expected);
}

#[test]
fn fix_closure_with_multiple_occurrences() {
    let source = r#"ls | where {|f| $f.size > 100kb and $f.type == "file"}"#;
    let expected = r#"ls | where { $it.size > 100kb and $it.type == "file"}"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_is(source, expected);
}

#[test]
fn fix_closure_with_regex() {
    let source = r#"ls | where {|f| $f.name =~ "Car"}"#;
    let expected = r#"ls | where { $it.name =~ "Car"}"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_is(source, expected);
}

#[test]
fn fix_explanation_mentions_it() {
    let source = r"[1, 2, 3] | where {|x| $x > 2}";
    RULE.assert_fix_explanation_contains(source, "$it");
}

#[test]
fn fix_explanation_mentions_parameter_name() {
    let source = r"[1, 2, 3] | where {|num| $num > 2}";
    RULE.assert_fix_explanation_contains(source, "$num");
}

#[test]
fn fix_explanation_mentions_row_condition() {
    let source = r"[1, 2, 3] | where {|x| $x > 2}";
    RULE.assert_fix_explanation_contains(source, "row condition");
}

#[test]
fn fix_closure_with_string_operation() {
    let source = r#"ls | where {|f| ($f.name | str downcase) =~ "readme"}"#;
    let expected = r#"ls | where { ($it.name | str downcase) =~ "readme"}"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_is(source, expected);
}

#[test]
fn fix_closure_with_math() {
    let source = r"[1, 2, 3] | where {|x| $x * 2 > 3}";
    let expected = r"[1, 2, 3] | where { $it * 2 > 3}";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_is(source, expected);
}

#[test]
fn fix_closure_with_date_comparison() {
    let source = r"ls | where {|f| $f.modified >= (date now) - 2wk}";
    let expected = r"ls | where { $it.modified >= (date now) - 2wk}";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_is(source, expected);
}

#[test]
fn fix_multiline_closure() {
    let source = r"
def filter [] {
    ls | where {|f| $f.size > 1kb}
}
";
    let expected = r"
def filter [] {
    ls | where { $it.size > 1kb}
}
";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_is(source, expected);
}

#[test]
fn fix_closure_with_not_operator() {
    let source = r"[1, 2, 3] | where {|x| not ($x == 2)}";
    let expected = r"[1, 2, 3] | where { not ($it == 2)}";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_is(source, expected);
}

#[test]
fn fix_filter_simple_closure() {
    let source = r"[1, 2, 3] | filter {|x| $x > 2}";
    let expected = r"[1, 2, 3] | filter { $it > 2}";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_is(source, expected);
}

#[test]
fn fix_filter_with_field_access() {
    let source = r"ls | filter {|f| $f.size > 100kb}";
    let expected = r"ls | filter { $it.size > 100kb}";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_is(source, expected);
}

#[test]
fn fix_does_not_replace_similar_variable_names() {
    let source = r"$new_servers | where {|n| ($new_servers | get -o $n) == null }";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "$new_servers");
    RULE.assert_fixed_contains(source, "get -o $it");
}

#[test]
fn fix_nested_closure_with_same_param_name() {
    // Edge case: nested closure with same parameter name
    // Both closures are detected, but only the outer one gets fixed in a single
    // pass (the inner one would be fixed in a subsequent run)
    let source = r"[1, 2, 3] | where {|x| ([] | where {|x| $x > 0} | length) > 0 or $x > 2}";
    RULE.assert_count(source, 2); // Both inner and outer where closures are detected
    // Only the outer closure is fixed in first pass
    RULE.assert_fixed_contains(source, "or $it > 2"); // Outer closure fixed
}

#[test]
fn fix_closure_with_utf8_characters() {
    // UTF-8 safety: ensure multi-byte characters are handled correctly
    let source = r#"["测试", "テスト", "🎉"] | where {|文字| $文字 == "测试"}"#;
    let expected = r#"["测试", "テスト", "🎉"] | where { $it == "测试"}"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_is(source, expected);
}
