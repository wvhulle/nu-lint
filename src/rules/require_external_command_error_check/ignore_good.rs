use super::rule;

#[test]
fn test_ignore_with_complete() {
    let good_code = r"
^git status | complete | if $in.exit_code != 0 { return }
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_with_try() {
    let good_code = r"
try { ^command | process }
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_single_external() {
    let good_code = r"
^git status
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_no_external() {
    let good_code = r"
ls | where size > 1kb
";
    rule().assert_ignores(good_code);
}
