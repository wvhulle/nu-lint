use super::RULE;

#[test]
fn test_export_main_in_script_detected() {
    let bad_code = r"#!/usr/bin/env nu
export def main [] {
    print 'Hello'
}
";
    RULE.assert_detects(bad_code);
    RULE.assert_count(bad_code, 1);
}

#[test]
fn test_export_main_subcommand_in_script_detected() {
    let bad_code = r#"#!/usr/bin/env nu
export def "main test" [] {
    print "Running tests"
}
"#;
    RULE.assert_detects(bad_code);
    RULE.assert_count(bad_code, 1);
}

#[test]
fn test_multiple_main_subcommands_in_script() {
    let bad_code = r#"#!/usr/bin/env nu
export def "main build" [] {
    print "Building"
}

export def "main test" [] {
    print "Testing"
}
"#;
    RULE.assert_count(bad_code, 2);
}

#[test]
fn test_export_main_with_params_in_script() {
    let bad_code = r"#!/usr/bin/env nu
export def main [name: string] {
    print $'Hello ($name)'
}
";
    RULE.assert_detects(bad_code);
}
