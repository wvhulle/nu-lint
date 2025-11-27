pub mod kebab_case_commands;
pub mod screaming_snake_constants;
pub mod snake_case_variables;

use super::sets::RuleSet;
use crate::rule::Rule;

pub const RULES: &[Rule] = &[
    kebab_case_commands::rule(),
    screaming_snake_constants::rule(),
    snake_case_variables::rule(),
];

pub const fn rule_set() -> RuleSet {
    RuleSet {
        name: "naming",
        explanation: "Linting rules for naming conventions",
        rules: RULES,
    }
}
