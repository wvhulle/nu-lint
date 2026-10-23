use super::RULE;

#[test]
fn ignore_other_predicates() {
    for code in [
        "^find . -maxdepth 1 -name '*.rs'",
        "^find . -newer stamp",
        "^find . -mtime -7",
        "^find . -size +10M",
        "^find . -path '*/src/*'",
        "^find . -iname '*.RS'",
        "^find . -name '*.rs' -o -name '*.toml'",
        "^find . -type l",
        "^find . -empty",
        "^find . -name '*.tmp' -delete",
        "^find . -name '*.rs' -exec wc -l {} +",
        "^find . -print0",
    ] {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_patterns_with_different_glob_meaning() {
    RULE.assert_ignores("^find . -name '{a,b}.txt'");
    RULE.assert_ignores("^find src* -name x");
}

#[test]
fn ignore_dynamic_arguments() {
    RULE.assert_ignores("let dir = 'src'; ^find $dir -name '*.rs'");
}

#[test]
fn ignore_builtin_find() {
    RULE.assert_ignores("[a b] | find a");
}
