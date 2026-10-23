use super::RULE;

#[test]
fn ignores_bare_glob_patterns() {
    let code = r#"ls *.txt"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_glob_in_non_glob_context() {
    // echo expects a string, not a glob pattern
    let code = r#"echo "*.txt""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_glob_in_print_command() {
    // print expects a string, not a glob
    let code = r#"print "file?.rs""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_without_glob_chars() {
    let code = r#"ls "file.txt""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_bare_word_without_glob_chars() {
    let code = r#"ls src/main.rs"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_interpolation_with_glob() {
    // String interpolation can't be a glob pattern
    let code = r#"ls $"*.txt""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_raw_string_with_glob() {
    // Raw strings are for literal content
    let code = r#"ls r#'*.txt'#"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_backtick_string() {
    // Backtick strings have special semantics
    let code = r#"ls `*.txt`"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_glob_in_list_literal() {
    // Not a direct argument to a command
    let code = r#"let files = ["*.txt", "*.rs"]"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_glob_in_variable_assignment() {
    // Command position - different context
    let code = r#"let pattern = "*.txt""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_glob_in_custom_function_without_glob_param() {
    // Custom functions without glob-typed parameters
    let code = r#"
        def my-func [path: string] {}
        my-func "*.txt"
    "#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_glob_in_record_literal() {
    let code = r#"let config = {pattern: "*.txt"}"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_glob_passed_to_string_param() {
    // str contains expects string, not glob
    let code = r#""test*.txt" | str contains "*""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_glob_command_with_quoted_pattern() {
    // glob command expects a STRING parameter, not a glob pattern
    // It processes the pattern internally, so quotes should stay
    let code = r#"glob "**/*.rs""#;
    RULE.assert_ignores(code);
}
