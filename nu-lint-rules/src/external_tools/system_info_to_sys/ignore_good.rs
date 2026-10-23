use super::RULE;

#[test]
fn ignore_forms_without_equivalent() {
    for code in [
        "^hostname new-name",
        "^hostname -f",
        "^hostname -I",
        "^uname",
        "^uname -s",
        "^uname -a",
        "^uname -v",
        "^uname -rm",
        "^uptime",
        "^free -m",
        "^free -s 1",
        "^df -h",
        "^who",
        "^users",
    ] {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_builtins() {
    RULE.assert_ignores("sys host | get hostname");
    RULE.assert_ignores("$nu.os-info.arch");
}
