use super::RULE;

#[test]
fn detect_simple_closure() {
    RULE.assert_detects(r"[1, 2, 3] | where {|x| $x > 2}");
}

#[test]
fn detect_closure_with_different_param_name() {
    RULE.assert_detects(r"[1, 2, 3] | where {|num| $num > 2}");
}

#[test]
fn detect_closure_with_line_param() {
    RULE.assert_detects(r#"open --raw file.txt | lines | where {|line| $line =~ "pattern"}"#);
}

#[test]
fn detect_closure_with_field_access() {
    RULE.assert_detects(r"ls | where {|f| $f.size > 100kb}");
}

#[test]
fn detect_closure_with_item_param() {
    RULE.assert_detects(r#"open data.json | where {|item| $item.field == "value"}"#);
}

#[test]
fn detect_closure_with_pipeline() {
    RULE.assert_detects(r"[1, 2, 3] | where {|x| ($x | str length) > 0}");
}

#[test]
fn detect_closure_with_external_variable() {
    let code = r"
let threshold = 2
[1, 2, 3] | where {|x| $x > $threshold}
";
    RULE.assert_detects(code);
}

#[test]
fn detect_closure_with_complex_condition() {
    RULE.assert_detects(r#"ls | where {|f| $f.size > 100kb and $f.type == "file"}"#);
}

#[test]
fn detect_closure_with_regex() {
    RULE.assert_detects(r#"ls | where {|f| $f.name =~ "Car"}"#);
}

#[test]
fn detect_closure_with_not_operator() {
    RULE.assert_detects(r"[1, 2, 3] | where {|x| not ($x == 2)}");
}

#[test]
fn detect_multiple_where_closures() {
    let code = r"
[1, 2, 3] | where {|x| $x > 1} | where {|y| $y < 3}
";
    RULE.assert_count(code, 2);
}

#[test]
fn detect_closure_in_function() {
    let code = r"
def filter_large [] {
    ls | where {|f| $f.size > 1kb}
}
";
    RULE.assert_detects(code);
}

#[test]
fn detect_closure_with_string_operation() {
    RULE.assert_detects(r#"ls | where {|f| ($f.name | str downcase) =~ "readme"}"#);
}

#[test]
fn detect_closure_with_math() {
    RULE.assert_detects(r"[1, 2, 3] | where {|x| $x * 2 > 3}");
}

#[test]
fn detect_closure_with_date_comparison() {
    RULE.assert_detects(r"ls | where {|f| $f.modified >= (date now) - 2wk}");
}

#[test]
fn detect_filter_simple_closure() {
    RULE.assert_detects(r"[1, 2, 3] | filter {|x| $x > 2}");
}

#[test]
fn detect_filter_with_field_access() {
    RULE.assert_detects(r"ls | filter {|f| $f.size > 100kb}");
}

#[test]
fn detect_filter_with_different_param_name() {
    RULE.assert_detects(r"[1, 2, 3] | filter {|num| $num > 2}");
}

#[test]
fn detect_filter_with_complex_condition() {
    RULE.assert_detects(r#"ls | filter {|f| $f.size > 100kb and $f.type == "file"}"#);
}

#[test]
fn detect_filter_with_pipeline() {
    RULE.assert_detects(r"[1, 2, 3] | filter {|x| ($x | str length) > 0}");
}
