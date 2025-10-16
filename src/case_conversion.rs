/// Shared case conversion utilities for naming convention rules
/// Re-exports from the heck crate for consistency
use heck::{ToKebabCase, ToShoutySnakeCase, ToSnakeCase};

#[must_use]
pub fn to_snake_case(s: &str) -> String {
    s.to_snake_case()
}

#[must_use]
pub fn to_screaming_snake(s: &str) -> String {
    s.to_shouty_snake_case()
}

#[must_use]
pub fn to_kebab_case(s: &str) -> String {
    s.to_kebab_case()
}
