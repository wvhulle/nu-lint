use super::rule;

#[test]
fn detects_external_find_with_name_pattern() {
    let source = r#"^find . -name "*.rs""#;
    rule().assert_count(source, 1);
    rule().assert_help_contains(source, "glob");
    rule().assert_help_contains(source, "ls");
}

#[test]
fn detects_find_without_arguments() {
    rule().assert_detects("^find");
}

#[test]
fn detects_find_with_path_only() {
    rule().assert_detects("^find .");
}

#[test]
fn detects_find_with_type_flag() {
    rule().assert_detects(r"^find . -type f");
}

#[test]
fn detects_find_with_size_flag() {
    rule().assert_detects(r"^find . -size +100k");
}

#[test]
fn detects_find_with_mtime_flag() {
    rule().assert_detects(r"^find . -mtime +30");
}

#[test]
fn detects_find_with_empty_flag() {
    rule().assert_detects(r"^find . -empty");
}
