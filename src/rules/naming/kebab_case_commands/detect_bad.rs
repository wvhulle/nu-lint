use super::rule;

#[test]
fn detect_camel_case_command() {
    let bad_code = r#"
def myCommand [] {
    print "bad naming"
}
"#;

    rule().assert_detects(bad_code);
    rule().assert_count(bad_code, 1);
}

#[test]
fn detect_underscore_command() {
    let bad_code = r#"
def my_command [] {
    print "underscore instead of hyphen"
}
"#;

    rule().assert_detects(bad_code);
    rule().assert_count(bad_code, 1);
}

#[test]
fn detect_pascal_case_command() {
    let bad_code = r#"
def AnotherCommand [] {
    print "CamelCase"
}
"#;

    rule().assert_detects(bad_code);
    rule().assert_count(bad_code, 1);
}
