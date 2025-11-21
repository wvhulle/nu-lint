use core::{
    fmt::{self, Display},
    hash::{Hash, Hasher},
};

use serde::Serialize;

use crate::LintLevel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleSet {
    pub name: &'static str,
    pub explanation: &'static str,
    pub rules: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleMap {
    pub name: &'static str,
    pub explanation: &'static str,
    pub rules: &'static [(&'static str, LintLevel)],
}

impl Serialize for RuleMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.name)
    }
}

const fn naming_rule_set() -> RuleSet {
    RuleSet {
        name: "naming",
        explanation: "Linting rules for naming conventions",
        rules: &[
            "kebab_case_commands",
            "screaming_snake_constants",
            "snake_case_variables",
        ],
    }
}

const fn formatting_rule_set() -> RuleSet {
    RuleSet {
        name: "formatting",
        explanation: "Check that code is formatted according to the official Nushell guidelines.",
        rules: &[
            "brace_spacing",
            "no_trailing_spaces",
            "omit_list_commas",
            "pipe_spacing",
            "prefer_multiline_functions",
            "prefer_multiline_lists",
            "prefer_multiline_records",
        ],
    }
}

const fn error_handling_rule_set() -> RuleSet {
    RuleSet {
        name: "error-handling",
        explanation: "Error handling best practices",
        rules: &[
            "add_metadata_to_error",
            "check_complete_exit_code",
            "descriptive_error_messages",
            "escape_string_interpolation_operators",
            "prefer_complete_for_external_commands",
            "prefer_error_make_for_stderr",
            "prefer_try_for_error_handling",
            "print_exit_use_error_make",
        ],
    }
}

const fn type_safety_rule_set() -> RuleSet {
    RuleSet {
        name: "type-safety",
        explanation: "Enforce explicit typing of variables and pipelines.",
        rules: &[
            "external_script_as_argument",
            "missing_type_annotation",
            "prefer_path_type",
            "typed_pipeline_io",
        ],
    }
}

const fn side_effects_rule_set() -> RuleSet {
    RuleSet {
        name: "side-effects",
        explanation: "Side effects (or effects) are things commands do that escape the type \
                      system, but happen often and may cause unexpected behavior.",
        rules: &[
            "mixed_io_types",
            "print_and_return_data",
            "pure_before_side_effects",
        ],
    }
}

const fn documentation_rule_set() -> RuleSet {
    RuleSet {
        name: "documentation",
        explanation: "Documentation quality rules",
        rules: &[
            "exported_function_docs",
            "descriptive_error_messages",
            "main_positional_args_docs",
            "main_named_args_docs",
        ],
    }
}

const fn performance_rule_set() -> RuleSet {
    RuleSet {
        name: "performance",
        explanation: "Performance optimization hints",
        rules: &[
            "prefer_builtin_cat",
            "prefer_builtin_echo",
            "prefer_builtin_find",
            "prefer_builtin_grep",
            "prefer_builtin_head",
            "prefer_builtin_http",
            "prefer_builtin_ls",
            "prefer_builtin_sed",
            "prefer_builtin_sort",
            "prefer_builtin_tail",
            "prefer_builtin_uniq",
            "prefer_compound_assignment",
            "prefer_direct_use",
            "prefer_is_not_empty",
            "prefer_lines_over_split",
            "prefer_nushell_over_jq",
            "prefer_parse_over_each_split",
            "prefer_parse_over_split_get",
            "prefer_pipeline_input",
            "prefer_range_iteration",
            "prefer_where_over_each_if",
            "prefer_where_over_for_if",
            "redundant_ignore",
            "remove_redundant_in",
            "unnecessary_variable_before_return",
        ],
    }
}

const fn systemd_rule_set() -> RuleSet {
    RuleSet {
        name: "systemd",
        explanation: "Rules for systemd service scripts",
        rules: &["systemd_journal_prefix"],
    }
}

pub const BUILTIN_LINT_SETS: &[(&str, RuleSet)] = &[
    ("documentation", documentation_rule_set()),
    ("error-handling", error_handling_rule_set()),
    ("formatting", formatting_rule_set()),
    ("naming", naming_rule_set()),
    ("performance", performance_rule_set()),
    ("side-effects", side_effects_rule_set()),
    ("systemd", systemd_rule_set()),
    ("type-safety", type_safety_rule_set()),
];

pub const DEFAULT_RULE_MAP: RuleMap = RuleMap {
    name: "default",
    explanation: "Default lint levels for all rules",
    rules: &[
        ("check_complete_exit_code", LintLevel::Warn),
        ("collapsible_if", LintLevel::Warn),
        ("dangerous_file_operations", LintLevel::Warn),
        ("descriptive_error_messages", LintLevel::Warn),
        ("add_metadata_to_error", LintLevel::Warn),
        ("escape_string_interpolation_operators", LintLevel::Deny),
        ("exit_only_in_main", LintLevel::Deny),
        ("unnecessary_ignore", LintLevel::Warn),
        ("exported_function_docs", LintLevel::Warn),
        ("main_positional_args_docs", LintLevel::Warn),
        ("main_named_args_docs", LintLevel::Warn),
        ("forbid_excessive_nesting", LintLevel::Warn),
        ("external_script_as_argument", LintLevel::Warn),
        ("inline_single_use_function", LintLevel::Warn),
        ("mixed_io_types", LintLevel::Warn),
        ("print_and_return_data", LintLevel::Warn),
        ("pure_before_side_effects", LintLevel::Warn),
        ("kebab_case_commands", LintLevel::Warn),
        ("max_function_body_length", LintLevel::Warn),
        ("max_positional_params", LintLevel::Warn),
        ("missing_type_annotation", LintLevel::Warn),
        ("typed_pipeline_io", LintLevel::Warn),
        ("prefer_multiline_functions", LintLevel::Warn),
        ("prefer_multiline_lists", LintLevel::Warn),
        ("prefer_multiline_records", LintLevel::Warn),
        ("no_trailing_spaces", LintLevel::Warn),
        ("nu_deprecated", LintLevel::Deny),
        ("nu_parse_error", LintLevel::Deny),
        ("omit_list_commas", LintLevel::Warn),
        ("prefer_complete_for_external_commands", LintLevel::Warn),
        ("prefer_builtin_cat", LintLevel::Warn),
        ("prefer_builtin_echo", LintLevel::Warn),
        ("prefer_builtin_find", LintLevel::Warn),
        ("prefer_builtin_grep", LintLevel::Warn),
        ("prefer_builtin_head", LintLevel::Warn),
        ("prefer_nushell_over_jq", LintLevel::Warn),
        ("prefer_builtin_http", LintLevel::Warn),
        ("prefer_builtin_ls", LintLevel::Warn),
        ("prefer_builtin_other", LintLevel::Warn),
        ("prefer_builtin_sed", LintLevel::Warn),
        ("prefer_builtin_sort", LintLevel::Warn),
        ("prefer_builtin_tail", LintLevel::Warn),
        ("prefer_builtin_uniq", LintLevel::Warn),
        ("prefer_compound_assignment", LintLevel::Warn),
        ("prefer_direct_use", LintLevel::Deny),
        ("prefer_error_make_for_stderr", LintLevel::Warn),
        ("prefer_try_for_error_handling", LintLevel::Warn),
        ("print_exit_use_error_make", LintLevel::Warn),
        ("prefer_is_not_empty", LintLevel::Warn),
        ("prefer_lines_over_split", LintLevel::Warn),
        ("prefer_match_over_if_chain", LintLevel::Warn),
        ("prefer_parse_over_split_get", LintLevel::Warn),
        ("prefer_parse_over_each_split", LintLevel::Warn),
        ("prefer_path_type", LintLevel::Warn),
        ("prefer_pipeline_input", LintLevel::Warn),
        ("prefer_range_iteration", LintLevel::Warn),
        ("prefer_where_over_each_if", LintLevel::Warn),
        ("prefer_where_over_for_if", LintLevel::Warn),
        ("remove_redundant_in", LintLevel::Warn),
        ("screaming_snake_constants", LintLevel::Warn),
        ("snake_case_variables", LintLevel::Warn),
        ("brace_spacing", LintLevel::Warn),
        ("pipe_spacing", LintLevel::Warn),
        ("systemd_journal_prefix", LintLevel::Allow),
        ("unnecessary_mut", LintLevel::Warn),
        ("unnecessary_variable_before_return", LintLevel::Warn),
        ("unused_helper_functions", LintLevel::Warn),
        ("redundant_ignore", LintLevel::Warn),
        ("missing_stdin_in_shebang", LintLevel::Deny),
    ],
};

impl Hash for RuleSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Display for RuleSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Hash for RuleMap {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Display for RuleMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
