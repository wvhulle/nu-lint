use super::rule;

#[test]
fn test_try_block_with_external_command() {
    let good_code = r"try { ^curl https://api.example.com }";
    rule().assert_ignores(good_code);
}

#[test]
fn test_do_block_without_error_prone_ops() {
    let good_code = r"do {
        let x = 42
        let y = $x + 10
        $y
    }";
    rule().assert_ignores(good_code);
}

#[test]
fn test_do_block_with_safe_operations() {
    let good_code = r#"do {
        print "Hello world"
        [1, 2, 3] | each { $in * 2 }
    }"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_try_block_with_file_operations() {
    let good_code = r"try {
        open config.json | from json
    }";
    rule().assert_ignores(good_code);
}

#[test]
fn test_do_block_for_variable_scoping() {
    let good_code = r#"do {
        let local_var = "scoped value"
        $env.TEMP_VAR = $local_var
        $local_var
    }"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_nested_try_catch() {
    let good_code = r#"try {
        ^git status
        open README.md
    } catch {
        print "Operation failed"
    }"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_do_block_with_closures() {
    let good_code = r#"do {
        let processor = {|x| $x | str upcase}
        "hello" | do $processor
    }"#;
    rule().assert_ignores(good_code);
}
