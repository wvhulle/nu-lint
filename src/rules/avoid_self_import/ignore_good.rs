use super::rule;

#[test]
fn test_ignore_use_other_module() {
    let good_code = r#"use std"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_use_relative_path() {
    let good_code = r#"use ./utils.nu"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_source_other_file() {
    let good_code = r#"source config.nu"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_direct_function_call() {
    let good_code = r#"
def helper [] { print "help" }
def main [] { helper }
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_match_dispatch() {
    let good_code = r#"
export def "cmd sub" [] { print "sub" }

def main [...args: string] {
    match ($args | first | default "") {
        "sub" => { cmd sub }
        _ => { print "usage" }
    }
}
"#;
    rule().assert_ignores(good_code);
}
