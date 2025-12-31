use super::RULE;

#[test]
fn test_main_without_export_in_script_ignored() {
    let good_code = r"#!/usr/bin/env nu
def main [] {
    print 'Hello'
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_main_subcommand_without_export_in_script_ignored() {
    let good_code = r#"#!/usr/bin/env nu
def "main test" [] {
    print "Running tests"
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_export_non_main_function_in_script_ignored() {
    let good_code = r"#!/usr/bin/env nu
export def helper [] {
    print 'Helper function'
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_mixed_functions_in_script() {
    let good_code = r"#!/usr/bin/env nu
def main [] {
    print 'Main'
}

export def helper [] {
    print 'Helper'
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_function_with_main_in_name_ignored() {
    let good_code = r"#!/usr/bin/env nu
export def get-main-config [] {
    {key: 'value'}
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_export_main_in_module_ignored() {
    // Without shebang, this is treated as a module where export def main is valid
    let good_code = r"
export def main [] {
    print 'Hello'
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_export_main_subcommand_in_module_ignored() {
    // Without shebang, module subcommands with export are valid
    let good_code = r#"
export def "main test" [] {
    print "Running tests"
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_module_with_multiple_exports_ignored() {
    // This is a typical module pattern - should not trigger
    let good_code = r"
export def main []: int -> int {
    $in + 1
}

export def by [amount: int]: int -> int {
    $in + $amount
}
";
    RULE.assert_ignores(good_code);
}
