use super::RULE;

#[test]
fn detect_system_information_commands() {
    for code in [
        "^hostname",
        "^uname -r",
        "^uname --machine",
        "^uname -n",
        "^uptime -s",
        "^uptime --pretty",
        "^free",
        "^free -h",
        "let host = (^hostname)",
    ] {
        RULE.assert_detects(code);
    }
}
