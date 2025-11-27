use super::rule;

#[test]
fn replaces_fd_pattern_with_ls_glob() {
    let source = r#"^fd "test""#;
    rule().assert_replacement_contains(source, "ls ./**/*test*");
}

#[test]
fn replaces_fd_with_directory() {
    let source = "^fd test src";
    rule().assert_replacement_contains(source, "ls src/**/*test*");
}

#[test]
fn replaces_fd_extension_with_glob() {
    let source = "^fd -e rs";
    rule().assert_replacement_contains(source, "ls ./**/*.rs");
}

#[test]
fn replaces_fd_type_file() {
    let source = "^fd -t f";
    rule().assert_replacement_contains(source, "ls ./**/* | where type == file");
}

#[test]
fn replaces_fd_type_directory() {
    let source = "^fd -t d";
    rule().assert_replacement_contains(source, "ls ./**/* | where type == dir");
}

#[test]
fn replaces_fd_glob_pattern() {
    let source = "^fd -g '*.rs' src";
    rule().assert_replacement_contains(source, "ls src/**/*.rs");
}

#[test]
fn replaces_fd_with_pattern_and_extension() {
    let source = "^fd -e rs test";
    rule().assert_replacement_contains(source, "ls ./**/*.rs");
}

#[test]
fn explains_structured_data_advantage() {
    let source = "^fd test";
    rule().assert_fix_explanation_contains(source, "structured data");
}

#[test]
fn explains_hidden_files_note() {
    let source = "^fd -H";
    rule().assert_fix_explanation_contains(source, "hidden");
    rule().assert_fix_explanation_contains(source, "ls -a");
}
