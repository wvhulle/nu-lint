use super::RULE;

#[test]
fn test_do_block_with_external_command() {
    let bad_code = r"do { ^curl https://api.example.com }";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_do_block_with_file_operations() {
    let bad_code = r"do { open config.json | from json }";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_do_block_with_save_operation() {
    let bad_code = r"do {
        let data = [1, 2, 3]
        $data | save output.json
    }";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_do_block_with_network_operation() {
    let bad_code = r#"do { http get "https://jsonplaceholder.typicode.com/posts/1" }"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_do_block_with_multiple_error_prone_ops() {
    let bad_code = r#"do {
        ^git status
        open README.md
        http get "https://api.github.com/repos/nushell/nushell"
    }"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_nested_do_with_external_command() {
    let bad_code = r#"
    def fetch-and-process [] {
        do {
            let result = (^curl -s "https://api.example.com")
            $result | from json
        }
    }"#;
    RULE.assert_detects(bad_code);
}
