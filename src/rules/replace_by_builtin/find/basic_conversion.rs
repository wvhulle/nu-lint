use crate::rules::replace_by_builtin::find::rule;

#[test]
fn detects_external_find_with_name_pattern() {
    let source = r#"^find . -name "*.rs""#;
    rule().assert_count(source, 1);
    rule().assert_help_contains(source, "glob");
    rule().assert_help_contains(source, "ls");
}

#[test]
fn replaces_find_name_with_ls_glob() {
    let source = r#"^find . -name "*.rs""#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "ls ./**/*.rs");
    rule().assert_fix_explanation_contains(source, "**");
    rule().assert_fix_explanation_contains(source, "subdirectories");
    rule().assert_fix_explanation_contains(source, "structured data");
}

#[test]
fn replaces_find_directory_traversal() {
    let source = "^find src";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "ls src/**/*");
    rule().assert_fix_explanation_contains(source, "recursive file search");
    rule().assert_fix_explanation_contains(source, "structured data");
}

#[test]
fn replaces_find_current_directory() {
    let source = "^find .";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "ls ./**/*");
}

#[test]
fn converts_complex_find_with_type_and_mtime() {
    let source = r"^find . -type f -mtime +30";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(
        source,
        "ls ./**/* | where type == file | where modified < ((date now) - 30day)",
    );
    rule().assert_fix_explanation_contains(source, "Pipeline filters replace find flags");
    rule().assert_fix_explanation_contains(source, "type:");
    rule().assert_fix_explanation_contains(source, "time:");
}

#[test]
fn converts_find_with_name_and_type() {
    let source = r#"^find /var/log -name "*.log" -type f"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "ls /var/log/**/*.log | where type == file");
    rule().assert_fix_explanation_contains(source, "**");
    rule().assert_fix_explanation_contains(source, "pattern");
    rule().assert_fix_explanation_contains(source, "where type == file");
}

#[test]
fn converts_find_with_size_filter() {
    let source = r"^find . -type f -size +100k";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(
        source,
        "ls ./**/* | where type == file | where size > 100kb",
    );
    rule().assert_fix_explanation_contains(source, "size:");
}

#[test]
fn converts_find_with_empty_flag() {
    let source = r"^find . -empty";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "ls ./**/* | where size == 0b");
    rule().assert_fix_explanation_contains(source, "empty:");
}

#[test]
fn ignores_builtin_find_for_data_filtering() {
    let source = r"ls | find toml";
    rule().assert_ignores(source);
}

#[test]
fn ignores_builtin_find_with_regex() {
    let source = r#"[abc bde arc abf] | find --regex "ab""#;
    rule().assert_ignores(source);
}

#[test]
fn ignores_builtin_find_on_strings() {
    let source = r"'Cargo.toml' | find cargo";
    rule().assert_ignores(source);
}

#[test]
fn detects_fd_simple_pattern() {
    let source = r#"^fd "*.rs""#;
    rule().assert_count(source, 1);
}

#[test]
fn detects_fd_with_directory() {
    let source = "^fd test src/";
    rule().assert_count(source, 1);
}

#[test]
fn distinguishes_bash_find_from_nushell_find() {
    let bash_find_source = r#"^find . -name "*.toml""#;
    let nushell_find_source = r"ls | find toml";
    rule().assert_count(bash_find_source, 1);
    rule().assert_ignores(nushell_find_source);
}

#[test]
fn explains_nushell_structured_data_advantage() {
    let source = r#"^find . -name "*.rs""#;
    rule().assert_count(source, 1);
}
