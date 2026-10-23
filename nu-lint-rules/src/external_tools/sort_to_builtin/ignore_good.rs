use super::RULE;

#[test]
fn ignore_untranslatable_flags() {
    for code in [
        "$text | ^sort -n",
        "$text | ^sort -k2",
        "$text | ^sort -t, -k 2,2",
        "$text | ^sort -h",
        "$text | ^sort -V",
        "$text | ^sort -R",
        "$text | ^sort -o out.txt",
    ] {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_merging_files() {
    RULE.assert_ignores("^sort a.txt b.txt");
    RULE.assert_ignores("^sort *.txt");
}

#[test]
fn ignore_reading_terminal() {
    RULE.assert_ignores("^sort");
}

#[test]
fn ignore_nushell_sort() {
    RULE.assert_ignores("$text | lines | sort");
}
