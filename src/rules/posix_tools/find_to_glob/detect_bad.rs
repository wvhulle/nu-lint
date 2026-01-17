use super::RULE;

#[test]
fn detects_external_find_with_name_pattern() {
    let source = r#"^find . -name "*.rs""#;
    RULE.assert_count(source, 1);
}

#[test]
fn detects_find_without_arguments() {
    RULE.assert_detects("^find");
}

#[test]
fn detects_find_with_path_only() {
    RULE.assert_detects("^find .");
}

#[test]
fn detects_find_with_type_flag() {
    RULE.assert_detects(r"^find . -type f");
}

#[test]
fn detects_find_with_size_flag() {
    RULE.assert_detects(r"^find . -size +100k");
}

#[test]
fn detects_find_with_mtime_flag() {
    RULE.assert_detects(r"^find . -mtime +30");
}

#[test]
fn detects_find_with_empty_flag() {
    RULE.assert_detects(r"^find . -empty");
}
