use super::rule;

#[test]
fn detect_simple_closure() {
    rule().assert_detects(r"[1, 2, 3] | where {|x| $x > 2}");
}

#[test]
fn detect_closure_with_different_param_name() {
    rule().assert_detects(r"[1, 2, 3] | where {|num| $num > 2}");
}

#[test]
fn detect_closure_with_line_param() {
    rule().assert_detects(r#"open --raw file.txt | lines | where {|line| $line =~ "pattern"}"#);
}

#[test]
fn detect_closure_with_field_access() {
    rule().assert_detects(r"ls | where {|f| $f.size > 100kb}");
}

#[test]
fn detect_closure_with_item_param() {
    rule().assert_detects(r#"open data.json | where {|item| $item.field == "value"}"#);
}

#[test]
fn detect_closure_with_pipeline() {
    rule().assert_detects(r"[1, 2, 3] | where {|x| ($x | str length) > 0}");
}

#[test]
fn detect_closure_with_external_variable() {
    let code = r"
let threshold = 2
[1, 2, 3] | where {|x| $x > $threshold}
";
    rule().assert_detects(code);
}

#[test]
fn detect_closure_with_complex_condition() {
    rule().assert_detects(r#"ls | where {|f| $f.size > 100kb and $f.type == "file"}"#);
}

#[test]
fn detect_closure_with_regex() {
    rule().assert_detects(r#"ls | where {|f| $f.name =~ "Car"}"#);
}

#[test]
fn detect_closure_with_not_operator() {
    rule().assert_detects(r"[1, 2, 3] | where {|x| not ($x == 2)}");
}

#[test]
fn detect_multiple_where_closures() {
    let code = r"
[1, 2, 3] | where {|x| $x > 1} | where {|y| $y < 3}
";
    rule().assert_violation_count_exact(code, 2);
}

#[test]
fn detect_closure_in_function() {
    let code = r"
def filter_large [] {
    ls | where {|f| $f.size > 1kb}
}
";
    rule().assert_detects(code);
}

#[test]
fn detect_closure_with_string_operation() {
    rule().assert_detects(r#"ls | where {|f| ($f.name | str downcase) =~ "readme"}"#);
}

#[test]
fn detect_closure_with_math() {
    rule().assert_detects(r"[1, 2, 3] | where {|x| $x * 2 > 3}");
}

#[test]
fn detect_closure_with_date_comparison() {
    rule().assert_detects(r"ls | where {|f| $f.modified >= (date now) - 2wk}");
}
