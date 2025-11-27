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

const fn bashisms_rule_set() -> RuleSet {
    RuleSet {
        name: "bashisms",
        explanation: "Replace common bash/POSIX commands with native Nushell equivalents",
        rules: &[
            "prefer_builtin_awk",
            "prefer_builtin_cat",
            "prefer_builtin_curl",
            "prefer_builtin_cut",
            "prefer_builtin_date",
            "prefer_builtin_echo",
            "prefer_builtin_find",
            "prefer_builtin_grep",
            "prefer_builtin_head",
            "prefer_builtin_hostname",
            "prefer_builtin_http",
            "prefer_builtin_ls",
            "prefer_builtin_man",
            "prefer_builtin_printenv",
            "prefer_builtin_read",
            "prefer_builtin_sed",
            "prefer_builtin_sort",
            "prefer_builtin_tail",
            "prefer_builtin_uniq",
            "prefer_builtin_wc",
            "prefer_builtin_which",
        ],
    }
}

const fn external_tools_rule_set() -> RuleSet {
    RuleSet {
        name: "external-tools",
        explanation: "Replace modern CLI tools with native Nushell equivalents",
        rules: &[
            "prefer_builtin_exa",
            "prefer_builtin_eza",
            "prefer_builtin_fd",
            "prefer_nushell_over_jq",
            "prefer_builtin_rg",
            "prefer_builtin_wget",
        ],
    }
}

const fn performance_rule_set() -> RuleSet {
    RuleSet {
        name: "performance",
        explanation: "Performance optimization hints",
        rules: &[
            "prefer_compound_assignment",
            "prefer_direct_use",
            "prefer_lines_over_split",
            "prefer_parse_over_each_split",
            "prefer_parse_over_split_get",
            "prefer_pipeline_input",
            "prefer_range_iteration",
            "prefer_where_over_each_if",
            "prefer_where_over_for_if",
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
    ("bashisms", bashisms_rule_set()),
    ("documentation", documentation_rule_set()),
    ("error-handling", error_handling_rule_set()),
    ("external-tools", external_tools_rule_set()),
    ("formatting", formatting_rule_set()),
    ("naming", naming_rule_set()),
    ("performance", performance_rule_set()),
    ("side-effects", side_effects_rule_set()),
    ("systemd", systemd_rule_set()),
    ("type-safety", type_safety_rule_set()),
];

/// Rules can receive an optional default lint level (overriding the default of
/// Warn)
pub const RULE_LEVEL_OVERRIDES: RuleMap = RuleMap {
    name: "default",
    explanation: "Default lint levels for all rules",
    rules: &[
        ("escape_string_interpolation_operators", LintLevel::Deny),
        ("exit_only_in_main", LintLevel::Deny),
        ("missing_stdin_in_shebang", LintLevel::Deny),
        ("nu_deprecated", LintLevel::Deny),
        ("nu_parse_error", LintLevel::Deny),
        ("prefer_direct_use", LintLevel::Deny),
        ("systemd_journal_prefix", LintLevel::Allow),
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
