use super::RULE;

#[test]
fn ignore_other_counts() {
    for code in [
        "$text | ^wc",
        "$text | ^wc -w",
        "$text | ^wc -c",
        "$text | ^wc -lw",
        "^wc -l a.txt b.txt",
        "^wc -l *.rs",
    ] {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_nushell_length() {
    RULE.assert_ignores("$text | lines | length");
}
