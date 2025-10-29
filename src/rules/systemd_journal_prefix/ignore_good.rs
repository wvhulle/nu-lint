use super::rule;

#[test]
fn test_ignore_print_with_info_prefix() {
    let good_code = r#"print "<6>Starting process""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_print_with_error_prefix() {
    let good_code = r#"print "<3>Error occurred""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_print_with_warning_prefix() {
    let good_code = r#"print "<4>Warning: deprecated feature""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_print_with_debug_prefix() {
    let good_code = r#"print "<7>Debug: entering function""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_echo_with_notice_prefix() {
    let good_code = r#"echo "<5>System notification""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_echo_with_info_prefix() {
    let good_code = r#"echo "<6>Information message""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_multiple_prints_with_prefixes() {
    let good_code = r#"
print "<6>Starting task"
print "<6>Task completed"
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_print_with_emergency_prefix() {
    let good_code = r#"print "<0>System emergency""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_print_with_alert_prefix() {
    let good_code = r#"print "<1>Alert condition""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_print_with_critical_prefix() {
    let good_code = r#"print "<2>Critical failure""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_mixed_levels() {
    let good_code = r#"
print "<6>Starting"
print "<4>Warning detected"
print "<3>Error occurred"
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_print_with_keyword_emerg() {
    let good_code = r#"print "<emerg>System emergency""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_print_with_keyword_alert() {
    let good_code = r#"print "<alert>Alert condition""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_print_with_keyword_crit() {
    let good_code = r#"print "<crit>Critical failure""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_print_with_keyword_err() {
    let good_code = r#"print "<err>Error occurred""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_print_with_keyword_warning() {
    let good_code = r#"print "<warning>Warning message""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_print_with_keyword_notice() {
    let good_code = r#"print "<notice>Notice message""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_print_with_keyword_info() {
    let good_code = r#"print "<info>Information message""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_print_with_keyword_debug() {
    let good_code = r#"print "<debug>Debug message""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_echo_with_keyword_info() {
    let good_code = r#"echo "<info>Starting process""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_mixed_keyword_and_numeric() {
    let good_code = r#"
print "<info>Starting"
print "<4>Warning detected"
print "<err>Error occurred"
"#;
    rule().assert_ignores(good_code);
}
