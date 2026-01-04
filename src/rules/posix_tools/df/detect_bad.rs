use super::RULE;

#[test]
fn test_detect_df_no_args() {
    RULE.assert_detects("^df");
}

#[test]
fn test_detect_df_human_readable() {
    RULE.assert_detects("^df -h");
}

#[test]
fn test_detect_df_all() {
    RULE.assert_detects("^df -a");
}

#[test]
fn test_detect_df_show_type() {
    RULE.assert_detects("^df -T");
}

#[test]
fn test_detect_df_with_path() {
    RULE.assert_detects("^df /home");
}

#[test]
fn test_detect_df_multiple_flags() {
    RULE.assert_detects("^df -h -T");
}
