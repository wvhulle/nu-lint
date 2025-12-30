use super::RULE;

#[test]
fn replaces_find_name_with_ls_glob() {
    let source = r#"^find . -name "*.rs""#;
    RULE.assert_fixed_contains(source, "ls ./**/*.rs");
    RULE.assert_fix_explanation_contains(source, "**");
    RULE.assert_fix_explanation_contains(source, "subdirectories");
    RULE.assert_fix_explanation_contains(source, "structured data");
}

#[test]
fn replaces_find_directory_traversal() {
    let source = "^find src";
    RULE.assert_fixed_contains(source, "ls src/**/*");
    RULE.assert_fix_explanation_contains(source, "recursive file search");
}

#[test]
fn handles_no_arguments() {
    RULE.assert_fixed_contains("^find", "ls ./**/*");
}

#[test]
fn converts_pattern_without_wildcard() {
    let source = r#"^find . -name "test""#;
    RULE.assert_fixed_contains(source, "ls ./**/*test*");
}

#[test]
fn preserves_glob_pattern_with_wildcards() {
    let source = r#"^find src -name "test_*.rs""#;
    RULE.assert_fixed_contains(source, "ls src/**/test_*.rs");
}

#[test]
fn handles_path_with_spaces() {
    let source = r#"^find "my dir" -name "*.txt""#;
    RULE.assert_fixed_contains(source, r#"ls "my dir"/**/*.txt"#);
}

#[test]
fn handles_absolute_path() {
    let source = r"^find /usr/local/bin -type f";
    RULE.assert_fixed_contains(source, "ls /usr/local/bin/**/* | where type == file");
}

#[test]
fn converts_type_file() {
    RULE.assert_fixed_contains(r"^find . -type f", "ls ./**/* | where type == file");
}

#[test]
fn converts_type_directory() {
    RULE.assert_fixed_contains(r"^find . -type d", "ls ./**/* | where type == dir");
}

#[test]
fn converts_type_symlink() {
    RULE.assert_fixed_contains(r"^find . -type l", "ls ./**/* | where type == symlink");
}

#[test]
fn converts_size_greater_than() {
    RULE.assert_fixed_contains(r"^find . -size +1M", "ls ./**/* | where size > 1mb");
}

#[test]
fn converts_size_less_than() {
    RULE.assert_fixed_contains(r"^find . -size -500k", "ls ./**/* | where size < 500kb");
}

#[test]
fn converts_size_exact() {
    RULE.assert_fixed_contains(r"^find . -size 1G", "ls ./**/* | where size == 1gb");
}

#[test]
fn converts_mtime_older_than() {
    RULE.assert_fixed_contains(
        r"^find . -mtime +7",
        "ls ./**/* | where modified < ((date now) - 7day)",
    );
}

#[test]
fn converts_mtime_newer_than() {
    RULE.assert_fixed_contains(
        r"^find . -mtime -3",
        "ls ./**/* | where modified > ((date now) - 3day)",
    );
}

#[test]
fn converts_empty_flag() {
    RULE.assert_fixed_contains(r"^find . -empty", "ls ./**/* | where size == 0b");
    RULE.assert_fix_explanation_contains(r"^find . -empty", "empty:");
}

#[test]
fn combines_name_and_type_filters() {
    let source = r#"^find /var/log -name "*.log" -type f"#;
    RULE.assert_fixed_contains(source, "ls /var/log/**/*.log | where type == file");
    RULE.assert_fix_explanation_contains(source, "where type == file");
}

#[test]
fn combines_multiple_filters_in_pipeline() {
    let source = r#"^find . -name "*.rs" -type f -size +100k -mtime -7"#;
    RULE.assert_fixed_contains(
        source,
        "ls ./**/*.rs | where type == file | where size > 100kb | where modified > ((date now) - \
         7day)",
    );
    RULE.assert_fix_explanation_contains(source, "type:");
    RULE.assert_fix_explanation_contains(source, "size:");
    RULE.assert_fix_explanation_contains(source, "time:");
}

#[test]
fn combines_type_and_empty() {
    RULE.assert_fixed_contains(
        r"^find . -type f -empty",
        "ls ./**/* | where type == file | where size == 0b",
    );
}

#[test]
fn ignores_unsupported_maxdepth_but_processes_name() {
    RULE.assert_fixed_contains(r"^find . -maxdepth 2 -name '*.rs'", "*.rs");
}

#[test]
fn ignores_unsupported_executable_flag() {
    RULE.assert_fixed_contains(r"^find . -executable", "ls ./**/*");
}
