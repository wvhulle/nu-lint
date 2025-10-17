use heck::ToSnakeCase;

use super::*;

#[test]
fn test_to_snake_case() {
    assert_eq!("myVariable".to_snake_case(), "my_variable");
    assert_eq!("MyVariable".to_snake_case(), "my_variable");
    assert_eq!("my_variable".to_snake_case(), "my_variable");
    assert_eq!("CONSTANT".to_snake_case(), "constant");
}

#[test]
fn test_valid_snake_case() {
    assert!(SnakeCaseVariables::is_valid_snake_case("my_variable"));
    assert!(SnakeCaseVariables::is_valid_snake_case("x"));
    assert!(SnakeCaseVariables::is_valid_snake_case("var_2"));
}
