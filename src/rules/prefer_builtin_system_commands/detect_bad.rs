use super::rule;
use crate::LintContext;

#[test]
fn test_detect_external_env() {
    let bad_code = "^env";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect external env command"
        );
    });
}

#[test]
fn test_detect_external_env_with_var() {
    let bad_code = "^env | grep HOME";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect external env in pipeline"
        );
    });
}

#[test]
fn test_detect_external_printenv() {
    let bad_code = "^printenv PATH";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect external printenv command"
        );
    });
}

#[test]
fn test_detect_external_date() {
    let bad_code = "^date";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect external date command"
        );
    });
}

#[test]
fn test_detect_external_man() {
    let bad_code = "^man ls";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect external man command"
        );
    });
}

#[test]
fn test_detect_external_read() {
    let bad_code = "^read -p \"Enter value: \"";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect external read command"
        );
    });
}

#[test]
fn test_detect_external_read_silent() {
    let bad_code = "^read -s -p \"Password: \"";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect external read with silent flag"
        );
    });
}

#[test]
fn test_detect_external_in_script() {
    let bad_code = "def get-system-info [] { ^uname -a; ^hostname; ^whoami }";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect external commands in custom function"
        );
    });
}

#[test]
fn test_detect_external_whoami() {
    let bad_code = "^whoami";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect external whoami command"
        );
    });
}

#[test]
fn test_detect_external_hostname() {
    let bad_code = "^hostname";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect external hostname command"
        );
    });
}

#[test]
fn test_detect_external_hostname_for_ip() {
    let bad_code = "^hostname -I";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect external hostname -I for IP"
        );
    });
}

#[test]
fn test_detect_external_uname() {
    let bad_code = "^uname -a";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect external uname command"
        );
    });
}

#[test]
fn test_detect_external_which() {
    let bad_code = "^which ls";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect external which command"
        );
    });
}

#[test]
fn test_detect_external_pwd() {
    let bad_code = "^pwd";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect external pwd command"
        );
    });
}

#[test]
fn test_detect_external_cd() {
    let bad_code = "^cd /tmp";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect external cd command"
        );
    });
}
