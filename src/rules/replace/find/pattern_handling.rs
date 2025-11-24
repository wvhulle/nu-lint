use crate::rules::replace::find::rule;

#[test]
fn converts_iname_case_insensitive() {
    let source = r#"^find . -iname "*.TXT""#;
    rule().assert_replacement_contains(source, "ls ./**/*.TXT");
}

#[test]
fn converts_pattern_without_wildcard() {
    let source = r#"^find . -name "test""#;
    rule().assert_replacement_contains(source, "ls ./**/*test*");
}

#[test]
fn preserves_glob_pattern_with_wildcards() {
    let source = r#"^find src -name "test_*.rs""#;
    rule().assert_replacement_contains(source, "ls src/**/test_*.rs");
}

#[test]
fn handles_path_with_spaces() {
    let source = r#"^find "my dir" -name "*.txt""#;
    rule().assert_replacement_contains(source, r#"ls "my dir"/**/*.txt"#);
}

#[test]
fn handles_absolute_path() {
    let source = r"^find /usr/local/bin -type f";
    rule().assert_replacement_contains(source, "ls /usr/local/bin/**/* | where type == file");
}
