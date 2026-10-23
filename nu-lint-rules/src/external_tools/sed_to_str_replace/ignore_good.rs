use super::RULE;

#[test]
fn ignore_first_occurrence_per_line() {
    RULE.assert_ignores("$text | ^sed 's/foo/bar/'");
}

#[test]
fn ignore_regex_patterns() {
    for code in [
        "$text | ^sed 's/a.b/X/g'",
        "$text | ^sed 's/^/> /g'",
        r"$text | ^sed 's/a\/b/c/g'",
        "$text | ^sed 's/x*/y/g'",
    ] {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_replacement_references() {
    RULE.assert_ignores("$text | ^sed 's/a/&&/g'");
    RULE.assert_ignores(r"$text | ^sed 's/a/\n/g'");
}

#[test]
fn ignore_other_scripts_and_flags() {
    for code in [
        "$text | ^sed '/debug/d'",
        "$text | ^sed -n '2p'",
        "$text | ^sed -E 's/(a+)/x/g'",
        "^sed -i 's/a/b/g' file.txt",
        "$text | ^sed -e 's/a/b/g' -e 's/c/d/g'",
        "$text | ^sed 's/a/b/gI'",
    ] {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_dynamic_script() {
    RULE.assert_ignores(r#"let from = "a"; $text | ^sed $"s/($from)/b/g""#);
}

#[test]
fn ignore_multiple_files() {
    RULE.assert_ignores("^sed 's/a/b/g' one.txt two.txt");
}
