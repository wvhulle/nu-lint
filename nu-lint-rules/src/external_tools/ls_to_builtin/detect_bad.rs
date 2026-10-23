use super::RULE;

#[test]
fn detect_external_ls() {
    for code in [
        "^ls",
        "^ls /tmp",
        "^ls src tests",
        "^ls -la",
        "^ls -ltr src",
        "^ls -S",
        "^ls -d src",
        "let entries = (^ls -a)",
    ] {
        RULE.assert_detects(code);
    }
}
