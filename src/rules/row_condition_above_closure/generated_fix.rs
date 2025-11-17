use super::rule;

#[test]
fn fix_simple_closure() {
    let source = r"[1, 2, 3] | where {|x| $x > 2}";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r"$it > 2");
}

#[test]
fn fix_closure_with_different_param() {
    let source = r"[1, 2, 3] | where {|num| $num > 2}";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r"$it > 2");
}

#[test]
fn fix_closure_with_line_param() {
    let source = r#"open --raw file.txt | lines | where {|line| $line =~ "pattern"}"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r#"$it =~ "pattern""#);
}

#[test]
fn fix_closure_with_field_access() {
    let source = r"ls | where {|f| $f.size > 100kb}";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r"$it.size > 100kb");
}

#[test]
fn fix_closure_with_item_param() {
    let source = r#"open data.json | where {|item| $item.field == "value"}"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r#"$it.field == "value""#);
}

#[test]
fn fix_closure_with_pipeline() {
    let source = r"[1, 2, 3] | where {|x| ($x | str length) > 0}";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r"($it | str length) > 0");
}

#[test]
fn fix_closure_with_external_variable() {
    let source = r"
let threshold = 2
[1, 2, 3] | where {|x| $x > $threshold}
";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r"$it > $threshold");
}

#[test]
fn fix_closure_with_complex_condition() {
    let source = r#"ls | where {|f| $f.size > 100kb and $f.type == "file"}"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r#"$it.size > 100kb and $it.type == "file""#);
}

#[test]
fn fix_closure_with_regex() {
    let source = r#"ls | where {|f| $f.name =~ "Car"}"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r#"$it.name =~ "Car""#);
}

#[test]
fn fix_explanation_mentions_it() {
    let source = r"[1, 2, 3] | where {|x| $x > 2}";
    rule().assert_count(source, 1);
    rule().assert_fix_explanation_contains(source, "$it");
}

#[test]
fn fix_explanation_mentions_parameter_name() {
    let source = r"[1, 2, 3] | where {|num| $num > 2}";
    rule().assert_count(source, 1);
    rule().assert_fix_explanation_contains(source, "$num");
}

#[test]
fn fix_explanation_mentions_row_condition() {
    let source = r"[1, 2, 3] | where {|x| $x > 2}";
    rule().assert_count(source, 1);
    rule().assert_fix_explanation_contains(source, "row condition");
}

#[test]
fn fix_closure_with_string_operation() {
    let source = r#"ls | where {|f| ($f.name | str downcase) =~ "readme"}"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r#"($it.name | str downcase) =~ "readme""#);
}

#[test]
fn fix_closure_with_math() {
    let source = r"[1, 2, 3] | where {|x| $x * 2 > 3}";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r"$it * 2 > 3");
}

#[test]
fn fix_closure_with_date_comparison() {
    let source = r"ls | where {|f| $f.modified >= (date now) - 2wk}";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r"$it.modified >= (date now) - 2wk");
}

#[test]
fn fix_multiple_occurrences_of_parameter() {
    let source = r#"ls | where {|f| $f.size > 100kb and $f.type == "file"}"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "$it.size");
    rule().assert_replacement_contains(source, "$it.type");
}

#[test]
fn fix_simple_closure_exact() {
    let source = r"[1, 2, 3] | where {|x| $x > 2}";
    let expected = "$it > 2";
    rule().assert_count(source, 1);
    rule().assert_replacement_is(source, expected);
}

#[test]
fn fix_closure_with_field_access_exact() {
    let source = r"ls | where {|f| $f.size > 100kb}";
    let expected = "$it.size > 100kb";
    rule().assert_count(source, 1);
    rule().assert_replacement_is(source, expected);
}

#[test]
fn fix_closure_with_pipeline_exact() {
    let source = r"[1, 2, 3] | where {|x| ($x | str length) > 0}";
    let expected = "($it | str length) > 0";
    rule().assert_count(source, 1);
    rule().assert_replacement_is(source, expected);
}

#[test]
fn fix_closure_with_complex_condition_exact() {
    let source = r#"ls | where {|f| $f.size > 100kb and $f.type == "file"}"#;
    let expected = r#"$it.size > 100kb and $it.type == "file""#;
    rule().assert_count(source, 1);
    rule().assert_replacement_is(source, expected);
}

#[test]
fn fix_closure_with_external_variable_exact() {
    let source = r"
let threshold = 2
[1, 2, 3] | where {|x| $x > $threshold}
";
    let expected = "$it > $threshold";
    rule().assert_count(source, 1);
    rule().assert_replacement_is(source, expected);
}

#[test]
fn fix_closure_with_string_operation_exact() {
    let source = r#"ls | where {|f| ($f.name | str downcase) =~ "readme"}"#;
    let expected = r#"($it.name | str downcase) =~ "readme""#;
    rule().assert_count(source, 1);
    rule().assert_replacement_is(source, expected);
}

#[test]
fn fix_multiline_closure_exact() {
    let source = r"
def filter [] {
    ls | where {|f| $f.size > 1kb}
}
";
    let expected = "$it.size > 1kb";
    rule().assert_count(source, 1);
    rule().assert_replacement_is(source, expected);
}

#[test]
fn fix_closure_with_regex_exact() {
    let source = r#"ls | where {|f| $f.name =~ "Car"}"#;
    let expected = r#"$it.name =~ "Car""#;
    rule().assert_count(source, 1);
    rule().assert_replacement_is(source, expected);
}

#[test]
fn fix_closure_with_not_operator_exact() {
    let source = r"[1, 2, 3] | where {|x| not ($x == 2)}";
    let expected = "not ($it == 2)";
    rule().assert_count(source, 1);
    rule().assert_replacement_is(source, expected);
}
