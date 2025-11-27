pub mod exported_function;
pub mod main_named_args;
pub mod main_positional_args;

use super::sets::RuleSet;
use crate::rule::Rule;

pub const RULES: &[Rule] = &[
    exported_function::rule(),
    super::descriptive_error_messages::rule(),
    main_positional_args::rule(),
    main_named_args::rule(),
];

pub const fn rule_set() -> RuleSet {
    RuleSet {
        name: "documentation",
        explanation: "Documentation quality rules",
        rules: RULES,
    }
}
