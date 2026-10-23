use super::RULE;

#[test]
fn test_ignore_simple_condition() {
    let good_code = r#"
if $flag { print "yes" }
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_negated_condition() {
    let good_code = r#"
if (not $flag) { print "no" }
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_variable_comparison() {
    let good_code = r#"
if $x == $expected { print "match" }
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_numeric_comparison() {
    let good_code = r#"
if $count == 0 { print "empty" }
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_string_comparison() {
    let good_code = r#"
if $name == "test" { print "is test" }
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_null_comparison() {
    let good_code = r#"
if $value == null { print "is null" }
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_two_variables() {
    let good_code = r#"
if $flag1 == $flag2 { print "same" }
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_function_result_comparison() {
    let good_code = r#"
if (is-empty $list) { print "empty" }
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_where_without_bool_literal() {
    let good_code = r#"
$items | where { $in.active }
"#;
    RULE.assert_ignores(good_code);
}
