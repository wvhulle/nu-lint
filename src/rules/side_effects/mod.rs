pub mod mixed_io_types;
pub mod print_and_return_data;
pub mod pure_before_side_effects;

use super::sets::RuleSet;
use crate::rule::Rule;

pub const RULES: &[Rule] = &[
    mixed_io_types::rule(),
    print_and_return_data::rule(),
    pure_before_side_effects::rule(),
];

pub const fn rule_set() -> RuleSet {
    RuleSet {
        name: "side-effects",
        explanation: "Side effects (or effects) are things commands do that escape the type \
                      system, but happen often and may cause unexpected behavior.",
        rules: RULES,
    }
}
