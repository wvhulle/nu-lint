use super::rule;

#[test]
fn test_detect_external_ls() {
    let bad_code = "^ls -la";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_cat() {
    let bad_code = "^cat config.toml";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_grep() {
    let bad_code = "^grep \"error\" logs.txt";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_grep_with_flags() {
    let bad_code = "^grep -i \"warning\" *.log";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_head() {
    let bad_code = "^head -n 5 file.txt";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_tail() {
    let bad_code = "^tail -n 10 file.txt";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_find() {
    let bad_code = "^find . -name \"*.rs\"";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_commands_in_pipelines() {
    let bad_code = "^ls -la | ^grep config";

    rule().assert_violation_count(bad_code, 2);
}

#[test]
fn test_detect_external_commands_in_function() {
    let bad_code = r#"
def git_files [] {
    ^find . -name "*.rs" | ^head -10
}
"#;

    rule().assert_violation_count(bad_code, 2);
}

#[test]
fn test_detect_external_cat_with_multiple_files() {
    let bad_code = "^cat README.md CHANGELOG.md";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_sort() {
    let bad_code = "^sort file.txt";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_uniq() {
    let bad_code = "^uniq file.txt";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_commands_in_completion_function() {
    let bad_code = r#"
def "nu-complete git branches" [] {
    ^cat .git/refs/heads/* | ^sort
}
"#;

    rule().assert_violation_count(bad_code, 2);
}

#[test]
fn test_detect_external_head_tail_with_different_syntax() {
    let bad_code = "^head -5 data.csv";

    rule().assert_detects(bad_code);
}
