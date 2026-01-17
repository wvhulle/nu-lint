use super::RULE;

#[test]
fn ignore_nushell_reverse() {
    RULE.assert_ignores("open --raw file.txt | lines | reverse");
}

#[test]
fn ignore_builtin_reverse() {
    RULE.assert_ignores("[1 2 3] | reverse");
}

#[test]
fn ignore_open_command() {
    RULE.assert_ignores("open file.txt");
}

#[test]
fn ignore_lines_reverse() {
    RULE.assert_ignores("$content | lines | reverse");
}
