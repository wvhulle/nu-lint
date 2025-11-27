use super::rule;

#[test]
fn replaces_find_name_with_ls_glob() {
    let source = r#"^find . -name "*.rs""#;
    rule().assert_replacement_contains(source, "ls ./**/*.rs");
    rule().assert_fix_explanation_contains(source, "**");
    rule().assert_fix_explanation_contains(source, "subdirectories");
    rule().assert_fix_explanation_contains(source, "structured data");
}

#[test]
fn replaces_find_directory_traversal() {
    let source = "^find src";
    rule().assert_replacement_contains(source, "ls src/**/*");
    rule().assert_fix_explanation_contains(source, "recursive file search");
}

#[test]
fn handles_no_arguments() {
    rule().assert_replacement_contains("^find", "ls ./**/*");
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

#[test]
fn converts_type_file() {
    rule().assert_replacement_contains(r"^find . -type f", "ls ./**/* | where type == file");
}

#[test]
fn converts_type_directory() {
    rule().assert_replacement_contains(r"^find . -type d", "ls ./**/* | where type == dir");
}

#[test]
fn converts_type_symlink() {
    rule().assert_replacement_contains(r"^find . -type l", "ls ./**/* | where type == symlink");
}

#[test]
fn converts_size_greater_than() {
    rule().assert_replacement_contains(r"^find . -size +1M", "ls ./**/* | where size > 1mb");
}

#[test]
fn converts_size_less_than() {
    rule().assert_replacement_contains(r"^find . -size -500k", "ls ./**/* | where size < 500kb");
}

#[test]
fn converts_size_exact() {
    rule().assert_replacement_contains(r"^find . -size 1G", "ls ./**/* | where size == 1gb");
}

#[test]
fn converts_mtime_older_than() {
    rule().assert_replacement_contains(
        r"^find . -mtime +7",
        "ls ./**/* | where modified < ((date now) - 7day)",
    );
}

#[test]
fn converts_mtime_newer_than() {
    rule().assert_replacement_contains(
        r"^find . -mtime -3",
        "ls ./**/* | where modified > ((date now) - 3day)",
    );
}

#[test]
fn converts_empty_flag() {
    rule().assert_replacement_contains(r"^find . -empty", "ls ./**/* | where size == 0b");
    rule().assert_fix_explanation_contains(r"^find . -empty", "empty:");
}

#[test]
fn combines_name_and_type_filters() {
    let source = r#"^find /var/log -name "*.log" -type f"#;
    rule().assert_replacement_contains(source, "ls /var/log/**/*.log | where type == file");
    rule().assert_fix_explanation_contains(source, "where type == file");
}

#[test]
fn combines_multiple_filters_in_pipeline() {
    let source = r#"^find . -name "*.rs" -type f -size +100k -mtime -7"#;
    rule().assert_replacement_contains(
        source,
        "ls ./**/*.rs | where type == file | where size > 100kb | where modified > ((date now) - \
         7day)",
    );
    rule().assert_fix_explanation_contains(source, "type:");
    rule().assert_fix_explanation_contains(source, "size:");
    rule().assert_fix_explanation_contains(source, "time:");
}

#[test]
fn combines_type_and_empty() {
    rule().assert_replacement_contains(
        r"^find . -type f -empty",
        "ls ./**/* | where type == file | where size == 0b",
    );
}

#[test]
fn ignores_unsupported_maxdepth_but_processes_name() {
    rule().assert_replacement_contains(r"^find . -maxdepth 2 -name '*.rs'", "*.rs");
}

#[test]
fn ignores_unsupported_executable_flag() {
    rule().assert_replacement_contains(r"^find . -executable", "ls ./**/*");
}
