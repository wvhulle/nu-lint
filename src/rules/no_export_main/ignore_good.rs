use super::RULE;

#[test]
fn test_main_without_export_ignored() {
    let good_code = r"
def main [] {
    print 'Hello'
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_main_subcommand_without_export_ignored() {
    let good_code = r#"
def "main test" [] {
    print "Running tests"
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_export_non_main_function_ignored() {
    let good_code = r"
export def helper [] {
    print 'Helper function'
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_mixed_functions() {
    let good_code = r"
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
fn test_function_with_main_in_middle() {
    let good_code = r"
export def get-main-config [] {
    {key: 'value'}
}
";
    RULE.assert_ignores(good_code);
}
