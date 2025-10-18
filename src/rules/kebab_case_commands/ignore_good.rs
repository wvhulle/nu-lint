use heck::ToKebabCase;

#[test]
fn test_to_kebab_case() {
    assert_eq!("myCommand".to_kebab_case(), "my-command");
    assert_eq!("my_command".to_kebab_case(), "my-command");
    assert_eq!("my-command".to_kebab_case(), "my-command");
}
