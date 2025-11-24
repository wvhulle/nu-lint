use super::rule;

#[test]
fn test_command_without_ignore() {
    let good_code = "ls";
    rule().assert_ignores(good_code);
}

#[test]
fn test_mkdir_with_ignore() {
    // mkdir produces no output (output type: nothing), so | ignore is acceptable
    // (though unnecessary, it's not redundant since there's no output to discard)
    let good_code = "mkdir /tmp/test | ignore";
    rule().assert_ignores(good_code);
}

#[test]
fn test_cd_with_ignore() {
    // cd produces no output (output type: nothing), so | ignore is acceptable
    let good_code = "cd /tmp | ignore";
    rule().assert_ignores(good_code);
}

#[test]
fn test_touch_with_ignore() {
    // touch produces no output (output type: nothing), so | ignore is acceptable
    let good_code = "touch /tmp/file.txt | ignore";
    rule().assert_ignores(good_code);
}

#[test]
fn test_command_with_proper_handling() {
    // Using try for error handling is acceptable
    let good_code = "try { ls }";
    rule().assert_ignores(good_code);
}

#[test]
fn test_command_storing_result() {
    // Storing the result is acceptable
    let good_code = "let files = ls";
    rule().assert_ignores(good_code);
}
