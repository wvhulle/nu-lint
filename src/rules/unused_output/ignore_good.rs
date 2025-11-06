use super::rule;

#[test]
fn test_mkdir_with_ignore() {
    // Commands with no output are handled by unnecessary_ignore rule
    let good_code = "mkdir /tmp/test | ignore";
    rule().assert_ignores(good_code);
}

#[test]
fn test_cd_with_ignore() {
    // Commands with no output are handled by unnecessary_ignore rule
    let good_code = "cd /tmp | ignore";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ls_without_ignore() {
    let good_code = "ls";
    rule().assert_ignores(good_code);
}

#[test]
fn test_echo_without_ignore() {
    let good_code = "echo 'hello'";
    rule().assert_ignores(good_code);
}
