use super::RULE;

#[test]
fn test_fix_removes_export_keyword() {
    let bad_code = r"#!/usr/bin/env nu
export def main [] {
    print 'Hello'
}
";
    RULE.assert_detects(bad_code);

    RULE.assert_fix_erases(bad_code, "export");
}

#[test]
fn test_fix_removes_export_from_subcommand() {
    let bad_code = r#"#!/usr/bin/env nu
export def "main test" [] {
    print "Testing"
}
"#;
    RULE.assert_detects(bad_code);

    RULE.assert_fix_erases(bad_code, "export");
}
