use super::RULE;

#[test]
fn test_ignore_no_echo() {
    let good_code = r#"print "hello world""#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_direct_value() {
    let good_code = r#""hello world""#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_direct_variable() {
    let good_code = r"$value";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_pipeline_without_echo() {
    let good_code = r"$var | str upcase";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_print_command() {
    let good_code = r#"print $"Hello ($name)""#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_other_commands() {
    let good_code = r"
ls | where size > 1kb
";
    RULE.assert_ignores(good_code);
}
