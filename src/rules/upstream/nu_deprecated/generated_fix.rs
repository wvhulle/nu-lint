use super::RULE;

#[test]
fn fix_get_ignore_errors_short_flag() {
    let source = "{a: 1} | get -i a";
    RULE.assert_fixed_contains(source, "get -o a");
}

#[test]
fn fix_get_ignore_errors_long_flag() {
    let source = "{a: 1} | get --ignore-errors a";
    RULE.assert_fixed_contains(source, "get --optional a");
}
