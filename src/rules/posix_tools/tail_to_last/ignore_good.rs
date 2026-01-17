use super::RULE;

#[test]
fn ignore_nushell_last() {
    RULE.assert_ignores("open file.txt | lines | last 10");
}

#[test]
fn ignore_builtin_last() {
    RULE.assert_ignores("[1 2 3] | last 2");
}

#[test]
fn ignore_watch_command() {
    RULE.assert_ignores("watch log.txt { open --raw log.txt | lines | last 20 }");
}

#[test]
fn ignore_open_command() {
    RULE.assert_ignores("open file.txt");
}

#[test]
fn ignore_lines_last() {
    RULE.assert_ignores("$content | lines | last 5");
}
