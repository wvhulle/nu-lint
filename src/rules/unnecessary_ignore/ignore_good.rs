use super::rule;

#[test]
fn test_mkdir_without_ignore() {
    // Commands that produce no output don't trigger the rule if they don't use |
    // ignore
    let good_code = "mkdir /tmp/test";
    rule().assert_ignores(good_code);
}

#[test]
fn test_cd_without_ignore() {
    // Commands that produce no output don't trigger the rule if they don't use |
    // ignore
    let good_code = "cd /tmp";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ls_with_ignore() {
    // Commands that produce output are handled by unused_output rule
    let good_code = "ls | ignore";
    rule().assert_ignores(good_code);
}

#[test]
fn test_external_command_with_ignore() {
    // External commands are handled by external_command_ignore rule
    let good_code = "^ls -la | ignore";
    rule().assert_ignores(good_code);
}
