use super::RULE;

#[test]
fn test_fix_suggests_null() {
    let bad_code = "let x = nothing";
    RULE.assert_help_contains(bad_code, "use 'null' instead");
}
