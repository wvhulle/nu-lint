use super::RULE;

#[test]
fn detect_short_flag_ls_all() {
    RULE.assert_detects("ls -a");
}

#[test]
fn detect_short_flag_ls_long() {
    RULE.assert_detects("ls -l");
}

#[test]
fn detect_short_flag_with_value() {
    RULE.assert_detects("str replace -r 'a' 'b'");
}

#[test]
fn detect_multiple_short_flags() {
    // Should detect both short flags
    RULE.assert_count("ls -a -l", 2);
}

#[test]
fn detect_short_flag_in_pipeline() {
    RULE.assert_detects("ls | where name =~ '.rs' | sort-by -r size");
}

#[test]
fn detect_in_function() {
    RULE.assert_detects(
        r#"
        def list-files [] {
            ls -a
        }
    "#,
    );
}

#[test]
fn detect_get_optional_short() {
    RULE.assert_detects("$record | get -o field");
}
