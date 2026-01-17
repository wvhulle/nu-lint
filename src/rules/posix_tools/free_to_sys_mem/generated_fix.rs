use super::RULE;

#[test]
fn test_fix_free_no_args() {
    RULE.assert_fixed_is("^free", "sys mem");
}

#[test]
fn test_fix_free_human_readable() {
    RULE.assert_fixed_is("^free -h", "sys mem");
}

#[test]
fn test_fix_free_megabytes() {
    RULE.assert_fixed_is("^free -m", "sys mem");
}

#[test]
fn test_fix_free_total() {
    RULE.assert_fixed_is("^free -t", "sys mem");
}
