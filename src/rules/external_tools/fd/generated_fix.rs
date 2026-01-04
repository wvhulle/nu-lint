use super::RULE;

#[test]
fn replaces_fd_pattern_with_glob() {
    RULE.assert_fixed_contains(r#"^fd "test""#, "glob ./**/*test*");
}

#[test]
fn replaces_fd_with_directory() {
    RULE.assert_fixed_contains("^fd test src", "glob src/**/*test*");
}

#[test]
fn replaces_fd_extension_with_glob() {
    RULE.assert_fixed_contains("^fd -e rs", "glob ./**/*.rs");
}

#[test]
fn replaces_fd_glob_pattern() {
    RULE.assert_fixed_contains("^fd -g '*.rs' src", "glob src/**/*.rs");
}

#[test]
fn replaces_fd_with_pattern_and_extension() {
    RULE.assert_fixed_contains("^fd -e rs test", "glob ./**/*.rs");
}

#[test]
fn replaces_fd_type_file_uses_ls() {
    RULE.assert_fixed_contains("^fd -t f", "ls ./**/* | where type == file");
}

#[test]
fn replaces_fd_type_directory_uses_ls() {
    RULE.assert_fixed_contains("^fd -t d", "ls ./**/* | where type == dir");
}

#[test]
fn replaces_fd_type_symlink_uses_ls() {
    RULE.assert_fixed_contains("^fd -t l", "ls ./**/* | where type == symlink");
}

#[test]
fn replaces_fd_pattern_and_type_uses_ls() {
    RULE.assert_fixed_contains("^fd -t f test", "ls ./**/*test* | where type == file");
}
