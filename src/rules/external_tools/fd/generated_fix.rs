use super::RULE;

#[test]
fn replaces_fd_pattern_with_glob() {
    let source = r#"^fd "test""#;
    RULE.assert_fixed_contains(source, "glob ./**/*test*");
}

#[test]
fn replaces_fd_with_directory() {
    let source = "^fd test src";
    RULE.assert_fixed_contains(source, "glob src/**/*test*");
}

#[test]
fn replaces_fd_extension_with_glob() {
    let source = "^fd -e rs";
    RULE.assert_fixed_contains(source, "glob ./**/*.rs");
}

#[test]
fn replaces_fd_type_file() {
    let source = "^fd -t f";
    RULE.assert_fixed_contains(source, "glob ./**/* | where type == file");
}

#[test]
fn replaces_fd_type_directory() {
    let source = "^fd -t d";
    RULE.assert_fixed_contains(source, "glob ./**/* | where type == dir");
}

#[test]
fn replaces_fd_glob_pattern() {
    let source = "^fd -g '*.rs' src";
    RULE.assert_fixed_contains(source, "glob src/**/*.rs");
}

#[test]
fn replaces_fd_with_pattern_and_extension() {
    let source = "^fd -e rs test";
    RULE.assert_fixed_contains(source, "glob ./**/*.rs");
}

#[test]
fn explains_structured_data_advantage() {
    let source = "^fd test";
    RULE.assert_fix_explanation_contains(source, "structured data");
}

#[test]
fn explains_hidden_files_note() {
    let source = "^fd -H";
    RULE.assert_fix_explanation_contains(source, "hidden");
}
