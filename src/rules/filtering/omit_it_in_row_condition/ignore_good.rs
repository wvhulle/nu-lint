use super::RULE;

#[test]
fn ignore_bare_it() {
    RULE.assert_ignores(r"[1, 2, 3] | where $it > 2");
}

#[test]
fn ignore_already_bare_field() {
    RULE.assert_ignores(r#"ls | where type == "dir""#);
}

#[test]
fn ignore_it_on_right_side() {
    let code = r#"ls | where "dir" == $it.type"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignore_closure() {
    RULE.assert_ignores(r#"ls | where {|x| $x.type == "dir"}"#);
}

#[test]
fn ignore_filter_command() {
    RULE.assert_ignores(r#"ls | filter {|it| $it.type == "dir"}"#);
}

#[test]
fn ignore_nested_field_access() {
    RULE.assert_ignores(r"open data.json | where $it.user.name == 'John'");
}

#[test]
fn ignore_it_in_subexpression() {
    RULE.assert_ignores(r"ls | where ($it.size | into string) =~ '100'");
}

#[test]
fn ignore_it_in_pipeline() {
    RULE.assert_ignores(r#"ls | where ($it.name | str downcase) == "readme""#);
}

#[test]
fn ignore_where_without_arguments() {
    RULE.assert_ignores(r"def test [] { where }");
}

#[test]
fn ignore_non_where_commands() {
    RULE.assert_ignores(r"ls | each {|it| $it.size > 100kb}");
}

#[test]
fn ignore_complex_expression() {
    RULE.assert_ignores(r"ls | where not ($it.size > 100kb)");
}

#[test]
fn ignore_field_in_math_operation() {
    RULE.assert_ignores(r"[{x: 1}] | where $it.x * 2 > 5");
}
