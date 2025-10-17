use heck::ToKebabCase;

use super::*;

#[test]
fn test_to_kebab_case() {
    assert_eq!("myCommand".to_kebab_case(), "my-command");
    assert_eq!("my_command".to_kebab_case(), "my-command");
    assert_eq!("my-command".to_kebab_case(), "my-command");
}

#[test]
fn test_valid_kebab_case() {
    assert!(KebabCaseCommands::is_valid_kebab_case("my-command"));
    assert!(KebabCaseCommands::is_valid_kebab_case("command"));
    assert!(KebabCaseCommands::is_valid_kebab_case("my-long-command"));
}
