use core::fmt::{self, Display};

use crate::rule::Rule;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleSet {
    pub name: &'static str,
    pub explanation: &'static str,
    pub rules: &'static [Rule],
}

const ERROR_HANDLING_RULES: &[Rule] = &[
    super::error_make_metadata::rule(),
    super::check_complete_exit_code::rule(),
    super::descriptive_error_messages::rule(),
    super::escape_string_interpolation_operators::rule(),
    super::non_final_failure_check::rule(),
    super::prefer_error_make_for_stderr::rule(),
    super::prefer_try_for_error_handling::rule(),
    super::print_exit_use_error_make::rule(),
];

const TYPE_SAFETY_RULES: &[Rule] = &[
    super::external_script_as_argument::rule(),
    super::strong_typing::argument::rule(),
    super::strong_typing::paths::rule(),
    super::strong_typing::pipeline::rule(),
];

const PERFORMANCE_RULES: &[Rule] = &[
    super::prefer_compound_assignment::rule(),
    super::prefer_direct_use::rule(),
    super::prefer_lines_over_split::rule(),
    super::prefer_parse_command::rule(),
    super::prefer_pipeline_input::rule(),
    super::prefer_range_iteration::rule(),
    super::prefer_where_over_each_if::rule(),
    super::prefer_where_over_for_if::rule(),
    super::remove_redundant_in::rule(),
    super::unnecessary_variable_before_return::rule(),
];

const SYSTEMD_RULES: &[Rule] = &[super::systemd_journal_prefix::rule()];

const fn error_handling_rule_set() -> RuleSet {
    RuleSet {
        name: "error-handling",
        explanation: "Error handling best practices",
        rules: ERROR_HANDLING_RULES,
    }
}

const fn type_safety_rule_set() -> RuleSet {
    RuleSet {
        name: "type-safety",
        explanation: "Enforce explicit typing of variables and pipelines.",
        rules: TYPE_SAFETY_RULES,
    }
}

const fn performance_rule_set() -> RuleSet {
    RuleSet {
        name: "performance",
        explanation: "Performance optimization hints",
        rules: PERFORMANCE_RULES,
    }
}

const fn systemd_rule_set() -> RuleSet {
    RuleSet {
        name: "systemd",
        explanation: "Rules for systemd service scripts",
        rules: SYSTEMD_RULES,
    }
}

pub const ALL_GROUPS: &[RuleSet] = &[
    super::posix_tools::rule_set(),
    super::documentation::rule_set(),
    error_handling_rule_set(),
    super::external_tools::rule_set(),
    super::spacing::rule_set(),
    super::naming::rule_set(),
    performance_rule_set(),
    super::side_effects::rule_set(),
    systemd_rule_set(),
    type_safety_rule_set(),
];

impl Display for RuleSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
