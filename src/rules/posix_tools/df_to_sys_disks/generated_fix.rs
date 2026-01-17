use super::RULE;

#[test]
fn test_fix_df_no_args() {
    RULE.assert_fixed_is("^df", "sys disks");
}

#[test]
fn test_fix_df_human_readable() {
    RULE.assert_fixed_is("^df -h", "sys disks");
}

#[test]
fn test_fix_df_show_type() {
    RULE.assert_fixed_is("^df -T", "sys disks | select name mount type total free");
}

#[test]
fn test_fix_df_with_path() {
    RULE.assert_fixed_is("^df /home", "sys disks | where mount == /home");
}
