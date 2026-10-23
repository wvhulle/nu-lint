use super::RULE;

#[test]
fn test_already_using_concat_assign() {
    let good_code = r#"$list ++= [$item]"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_different_variables() {
    let good_code = r#"$list1 = ($list2 | append $x)"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_different_variables_no_parens() {
    let good_code = r#"$list1 = $list2 | append $x"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_chained_appends() {
    let good_code = r#"$list | append $x | append $y"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_append_with_extra_pipeline_steps() {
    let good_code = r#"$list = ($list | append $x | sort)"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_append_not_in_assignment() {
    let good_code = r#"$list | append $x"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_simple_assignment_no_pipeline() {
    let good_code = r#"$list = $other_list"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_assignment_with_different_command() {
    let good_code = r#"$list = ($list | filter {|x| $x > 0})"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_empty_pipeline() {
    let good_code = r#"$list = ()"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_append_in_function_call() {
    let good_code = r#"
def process [] {
    $in | append "suffix"
}
"#;
    RULE.assert_ignores(good_code);
}
