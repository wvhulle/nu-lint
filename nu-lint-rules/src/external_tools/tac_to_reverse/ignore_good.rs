use super::RULE;

#[test]
fn ignore_nushell_reverse() {
    RULE.assert_ignores("open --raw file.txt | lines | reverse");
}

#[test]
fn ignore_custom_separator() {
    RULE.assert_ignores("$text | ^tac -s ,");
}

#[test]
fn ignore_multiple_files() {
    RULE.assert_ignores("^tac file1.txt file2.txt");
}

#[test]
fn ignore_reading_terminal() {
    RULE.assert_ignores("^tac");
}
