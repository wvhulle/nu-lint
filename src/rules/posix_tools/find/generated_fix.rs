use super::RULE;

#[test]
fn replaces_find_name_with_glob() {
    RULE.assert_fixed_contains(r#"^find . -name "*.rs""#, "glob ./**/*.rs");
}

#[test]
fn replaces_find_directory_traversal() {
    RULE.assert_fixed_contains("^find src", "glob src/**/*");
}

#[test]
fn handles_no_arguments() {
    RULE.assert_fixed_contains("^find", "glob ./**/*");
}

#[test]
fn converts_pattern_without_wildcard() {
    RULE.assert_fixed_contains(r#"^find . -name "test""#, "glob ./**/*test*");
}

#[test]
fn preserves_glob_pattern_with_wildcards() {
    RULE.assert_fixed_contains(r#"^find src -name "test_*.rs""#, "glob src/**/test_*.rs");
}

#[test]
fn handles_path_with_spaces() {
    RULE.assert_fixed_contains(r#"^find "my dir" -name "*.txt""#, r#"glob "my dir"/**/*.txt"#);
}

#[test]
fn converts_type_file_uses_ls() {
    RULE.assert_fixed_contains(r"^find . -type f", "ls ./**/* | where type == file");
}

#[test]
fn converts_type_directory_uses_ls() {
    RULE.assert_fixed_contains(r"^find . -type d", "ls ./**/* | where type == dir");
}

#[test]
fn converts_type_symlink_uses_ls() {
    RULE.assert_fixed_contains(r"^find . -type l", "ls ./**/* | where type == symlink");
}

#[test]
fn converts_size_greater_than_uses_ls() {
    RULE.assert_fixed_contains(r"^find . -size +1M", "ls ./**/* | where size > 1mb");
}

#[test]
fn converts_size_less_than_uses_ls() {
    RULE.assert_fixed_contains(r"^find . -size -500k", "ls ./**/* | where size < 500kb");
}

#[test]
fn converts_size_exact_uses_ls() {
    RULE.assert_fixed_contains(r"^find . -size 1G", "ls ./**/* | where size == 1gb");
}

#[test]
fn converts_mtime_older_than_uses_ls() {
    RULE.assert_fixed_contains(
        r"^find . -mtime +7",
        "ls ./**/* | where modified < ((date now) - 7day)",
    );
}

#[test]
fn converts_mtime_newer_than_uses_ls() {
    RULE.assert_fixed_contains(
        r"^find . -mtime -3",
        "ls ./**/* | where modified > ((date now) - 3day)",
    );
}

#[test]
fn converts_empty_flag_uses_ls() {
    RULE.assert_fixed_contains(r"^find . -empty", "ls ./**/* | where size == 0b");
}

#[test]
fn combines_name_and_type_uses_ls() {
    RULE.assert_fixed_contains(
        r#"^find /var/log -name "*.log" -type f"#,
        "ls /var/log/**/*.log | where type == file",
    );
}

#[test]
fn combines_multiple_filters_uses_ls() {
    RULE.assert_fixed_contains(
        r#"^find . -name "*.rs" -type f -size +100k -mtime -7"#,
        "ls ./**/*.rs | where type == file | where size > 100kb | where modified > ((date now) - 7day)",
    );
}

#[test]
fn combines_type_and_empty_uses_ls() {
    RULE.assert_fixed_contains(
        r"^find . -type f -empty",
        "ls ./**/* | where type == file | where size == 0b",
    );
}

#[test]
fn handles_absolute_path_with_type_uses_ls() {
    RULE.assert_fixed_contains(
        r"^find /usr/local/bin -type f",
        "ls /usr/local/bin/**/* | where type == file",
    );
}

#[test]
fn ignores_unsupported_maxdepth_but_processes_name() {
    RULE.assert_fixed_contains(r"^find . -maxdepth 2 -name '*.rs'", "*.rs");
}

#[test]
fn ignores_unsupported_executable_flag() {
    RULE.assert_fixed_contains(r"^find . -executable", "glob ./**/*");
}
