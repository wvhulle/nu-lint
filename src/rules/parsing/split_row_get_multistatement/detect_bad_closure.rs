use super::RULE;

#[test]
fn test_detect_in_function_definition() {
    let bad_code = r#"
def process_data [] {
    let split = ("a:b:c" | split row ":")
    $split | get 0
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_in_closure() {
    let bad_code = r#"
let process = {||
    let split = ("hello world" | split row " ")
    $split | get 1
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_in_nested_closure() {
    let bad_code = r#"
["data:one", "data:two"] | each {|item|
    let split = ($item | split row ":")
    $split | get 1
}
"#;
    RULE.assert_detects(bad_code);
}
