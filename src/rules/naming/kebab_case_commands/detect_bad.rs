use super::RULE;

#[test]
fn detect_camel_case_command() {
    let bad_code = r#"
def myCommand [] {
    print "bad naming"
}
"#;

    RULE.assert_detects(bad_code);
    RULE.assert_count(bad_code, 1);
}

#[test]
fn detect_underscore_command() {
    let bad_code = r#"
def my_command [] {
    print "underscore instead of hyphen"
}
"#;

    RULE.assert_detects(bad_code);
    RULE.assert_count(bad_code, 1);
}

#[test]
fn detect_pascal_case_command() {
    let bad_code = r#"
def AnotherCommand [] {
    print "CamelCase"
}
"#;

    RULE.assert_detects(bad_code);
    RULE.assert_count(bad_code, 1);
}

#[test]
fn detect_bad_naming_in_subcommand() {
    let bad_code = r#"
def "tests myBadCommand" [] {
    print "subcommand with bad naming"
}
"#;

    RULE.assert_detects(bad_code);
    RULE.assert_count(bad_code, 1);
}
