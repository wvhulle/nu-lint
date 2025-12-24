use super::RULE;

#[test]
fn ignore_row_condition_with_it() {
    RULE.assert_ignores(r"[1, 2, 3] | where $it > 2");
}

#[test]
fn ignore_field_shorthand() {
    RULE.assert_ignores(r"ls | where size > 100kb");
}

#[test]
fn ignore_row_condition_with_field_access() {
    RULE.assert_ignores(r"ls | where $it.size > 100kb");
}

#[test]
fn ignore_row_condition_with_pipeline() {
    RULE.assert_ignores(r#"ls | where ($it.name | str downcase) =~ "readme""#);
}

#[test]
fn ignore_row_condition_with_external_var() {
    let code = r"
let threshold = 2
[1, 2, 3] | where $it > $threshold
";
    RULE.assert_ignores(code);
}

#[test]
fn ignore_stored_closure() {
    let code = r"
let cond = {|x| $x > 2}
[1, 2, 3] | where $cond
";
    RULE.assert_ignores(code);
}

#[test]
fn ignore_row_condition_with_complex_expression() {
    RULE.assert_ignores(r"[1, 2, 3] | where ($it | into string | str length) > 0");
}

#[test]
fn ignore_row_condition_with_comparison() {
    RULE.assert_ignores(r#"ls | where name =~ "Car""#);
}

#[test]
fn ignore_row_condition_with_date() {
    RULE.assert_ignores(r"ls | where modified >= (date now) - 2wk");
}

#[test]
fn ignore_other_commands_with_closures() {
    RULE.assert_ignores(r"[1, 2, 3] | each {|x| $x * 2}");
}

#[test]
fn ignore_filter_command_with_closure() {
    RULE.assert_ignores(r"[1, 2, 3] | filter {|x| $x > 2}");
}

#[test]
fn ignore_row_condition_in_parens() {
    RULE.assert_ignores(r"[1, 2, 3] | where ($it > 2)");
}

#[test]
fn ignore_where_without_arguments() {
    RULE.assert_ignores(r"def test [] { where }");
}
