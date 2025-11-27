pub mod brace_spacing;
pub mod no_trailing_spaces;
pub mod omit_list_commas;
pub mod pipe_spacing;
pub mod prefer_multiline_functions;
pub mod prefer_multiline_lists;
pub mod prefer_multiline_records;

use super::sets::RuleSet;
use crate::rule::Rule;

pub const RULES: &[Rule] = &[
    brace_spacing::rule(),
    no_trailing_spaces::rule(),
    omit_list_commas::rule(),
    pipe_spacing::rule(),
    prefer_multiline_functions::rule(),
    prefer_multiline_lists::rule(),
    prefer_multiline_records::rule(),
];

pub const fn rule_set() -> RuleSet {
    RuleSet {
        name: "formatting",
        explanation: "Check that code is formatted according to the official Nushell guidelines.",
        rules: RULES,
    }
}
