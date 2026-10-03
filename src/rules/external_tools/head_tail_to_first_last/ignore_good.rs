use super::RULE;

#[test]
fn ignore_untranslatable_flags() {
    for code in [
        "^tail -f app.log",
        "^tail -F app.log",
        "^head -c 5 file.bin",
        "^tail -q -n 2 a.txt",
        "$text | ^head -z",
    ] {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_negative_counts() {
    RULE.assert_ignores("$text | ^head -n -2");
}

#[test]
fn ignore_multiple_files_and_globs() {
    RULE.assert_ignores("^head a.txt b.txt");
    RULE.assert_ignores("^tail -n 1 *.log");
}

#[test]
fn ignore_reading_terminal() {
    RULE.assert_ignores("^head -n 1");
}

#[test]
fn ignore_nushell_selection() {
    RULE.assert_ignores("open --raw log.txt | lines | first 5");
    RULE.assert_ignores("ls | last 3");
}
