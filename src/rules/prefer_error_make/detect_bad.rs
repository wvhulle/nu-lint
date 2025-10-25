use super::rule;

#[test]
fn test_detect_print_exit_pattern() {
    let bad_code = r#"
def bad-error [] {
    print "Error occurred"
    exit 1
}
"#;
    rule().assert_violation_count_exact(bad_code, 1);
}
