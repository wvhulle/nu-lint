use super::rule;
use crate::LintContext;

#[test]
fn test_detect_manual_string_splitting_device() {
    let bad_code = r#"
let line = "Device AA:BB:CC:DD:EE:FF MyDevice"
let parts = ($line | split row " ")
let mac = ($parts | get 1)
let name = ($parts | skip 2 | str join " ")
"#;

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect manual string splitting for device info"
        );
    });
}

#[test]
fn test_detect_manual_string_splitting_user_data() {
    let bad_code = r#"
let data = "user:john:1000"
let fields = ($data | split row ":")
let username = ($fields | get 0)
"#;

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect manual string splitting for user data"
        );
    });
}
