use super::rule;

#[test]
fn test_detect_external_env() {
    let bad_code = "^env";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_env_with_var() {
    let bad_code = "^env | grep HOME";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_printenv() {
    let bad_code = "^printenv PATH";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_date() {
    let bad_code = "^date";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_man() {
    let bad_code = "^man ls";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_read() {
    let bad_code = "^read -p \"Enter value: \"";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_read_silent() {
    let bad_code = "^read -s -p \"Password: \"";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_in_script() {
    let bad_code = "def get-system-info [] { ^uname -a; ^hostname; ^whoami }";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_whoami() {
    let bad_code = "^whoami";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_hostname() {
    let bad_code = "^hostname";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_hostname_for_ip() {
    let bad_code = "^hostname -I";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_uname() {
    let bad_code = "^uname -a";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_which() {
    let bad_code = "^which ls";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_pwd() {
    let bad_code = "^pwd";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_cd() {
    let bad_code = "^cd /tmp";

    rule().assert_violation_count_exact(bad_code, 1);
}
