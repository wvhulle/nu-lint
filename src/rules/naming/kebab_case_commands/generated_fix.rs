use super::rule;

#[test]
fn test_kebab_case_fix_camel_case() {
    let bad_code = "def myCommand [] { echo \"test\" }";
    rule().assert_replacement_contains(bad_code, "my-command");
}

#[test]
fn test_kebab_case_fix_snake_case() {
    let bad_code = "def my_command [] { echo \"test\" }";
    rule().assert_replacement_contains(bad_code, "my-command");
}

#[test]
fn test_kebab_case_fix_pascal_case() {
    let bad_code = "def MyCommand [] { echo \"test\" }";
    rule().assert_replacement_contains(bad_code, "my-command");
}

#[test]
fn test_kebab_case_fix_export_def() {
    let bad_code = "export def myCommand [] { echo \"test\" }";
    rule().assert_replacement_contains(bad_code, "my-command");
}

#[test]
fn test_kebab_case_fix_multiple_commands() {
    let bad_code = r#"
def firstCommand [] { echo "first" }
def secondCommand [] { echo "second" }
"#;
    rule().assert_count(bad_code, 2);
    rule().assert_replacement_contains(bad_code, "first-command");
}
