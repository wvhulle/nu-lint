use super::RULE;

#[test]
fn ignore_formatting_flags() {
    for code in ["^cat -n file.txt", "^cat -A file.txt", "^cat -s file.txt"] {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_concatenation() {
    RULE.assert_ignores("^cat a.txt b.txt");
    RULE.assert_ignores("^cat *.txt");
}

#[test]
fn ignore_stdin_passthrough() {
    RULE.assert_ignores("$text | ^cat");
    RULE.assert_ignores("^cat");
    RULE.assert_ignores("$text | ^cat - footer.txt");
}

#[test]
fn ignore_nushell_open() {
    RULE.assert_ignores("open --raw file.txt");
}
