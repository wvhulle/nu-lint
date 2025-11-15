use core::{
    fmt::{self, Display},
    hash::{Hash, Hasher},
};
use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use serde::Serialize;

use crate::LintLevel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleSet {
    pub name: String,
    pub description: String,
    pub rules: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleMap {
    pub name: String,
    pub description: String,
    pub rules: HashMap<String, LintLevel>,
}

impl Serialize for RuleMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.name)
    }
}

fn naming_rule_set() -> RuleSet {
    RuleSet {
        name: "naming".to_string(),
        description: "Linting rules for naming conventions".to_string(),
        rules: HashSet::from([
            "snake_case_variables".to_string(),
            "kebab_case_commands".to_string(),
            "screaming_snake_constants".to_string(),
        ]),
    }
}

fn idioms_rule_set() -> RuleSet {
    RuleSet {
        name: "idioms".to_string(),
        description: "Linting rules for idiomatic Nushell expressions".to_string(),
        rules: HashSet::from([
            "prefer_builtin_ls".to_string(),
            "prefer_parse_command".to_string(),
            "prefer_where_over_each_if".to_string(),
            "prefer_builtin_uniq".to_string(),
            "prefer_builtin_http".to_string(),
            "prefer_builtin_grep".to_string(),
            "remove_redundant_in".to_string(),
            "prefer_builtin_find".to_string(),
            "prefer_lines_over_split".to_string(),
            "prefer_builtin_cat".to_string(),
            "prefer_compound_assignment".to_string(),
            "unnecessary_ignore".to_string(),
            "prefer_parse_over_each_split".to_string(),
            "prefer_builtin_sed".to_string(),
            "prefer_match_over_if_chain".to_string(),
            "prefer_where_over_for_if".to_string(),
            "prefer_builtin_head".to_string(),
            "prefer_pipeline_input".to_string(),
            "prefer_builtin_sort".to_string(),
            "prefer_builtin_tail".to_string(),
            "prefer_builtin_other".to_string(),
            "never_use_echo".to_string(),
            "prefer_is_not_empty".to_string(),
            "prefer_range_iteration".to_string(),
        ]),
    }
}

fn pedantic_rule_set() -> RuleSet {
    RuleSet {
        name: "pedantic".to_string(),
        description: "Strict linting for high code quality standards".to_string(),
        rules: HashSet::from([
            "forbid_excessive_nesting".to_string(),
            "max_function_body_length".to_string(),
            "max_positional_params".to_string(),
            "inline_single_use_function".to_string(),
            "unnecessary_variable_before_return".to_string(),
            "collapsible_if".to_string(),
            "unnecessary_mut".to_string(),
            "unused_helper_functions".to_string(),
            "prefer_direct_use".to_string(),
        ]),
    }
}

fn formatting_rule_set() -> RuleSet {
    RuleSet {
        name: "formatting".to_string(),
        description: "Code formatting and style rules".to_string(),
        rules: HashSet::from([
            "no_trailing_spaces".to_string(),
            "prefer_multiline_lists".to_string(),
            "pipe_spacing".to_string(),
            "prefer_multiline_records".to_string(),
            "omit_list_commas".to_string(),
            "prefer_multiline_functions".to_string(),
            "brace_spacing".to_string(),
        ]),
    }
}

fn error_handling_rule_set() -> RuleSet {
    RuleSet {
        name: "error-handling".to_string(),
        description: "Error handling best practices".to_string(),
        rules: HashSet::from([
            "prefer_error_make_for_stderr".to_string(),
            "descriptive_error_messages".to_string(),
            "print_exit_use_error_make".to_string(),
            "escape_string_interpolation_operators".to_string(),
            "pipeline_handle_errors".to_string(),
            "check_complete_exit_code".to_string(),
            "error_make_metadata".to_string(),
            "dangerous_file_operations".to_string(),
        ]),
    }
}

fn type_safety_rule_set() -> RuleSet {
    RuleSet {
        name: "type-safety".to_string(),
        description: "Type annotation and safety rules".to_string(),
        rules: HashSet::from([
            "missing_type_annotation".to_string(),
            "prefer_path_type".to_string(),
            "typed_pipeline_io".to_string(),
        ]),
    }
}

fn code_quality_rule_set() -> RuleSet {
    RuleSet {
        name: "code-quality".to_string(),
        description: "General code quality and maintainability rules".to_string(),
        rules: HashSet::from([
            "forbid_excessive_nesting".to_string(),
            "prefer_direct_use".to_string(),
            "inline_single_use_function".to_string(),
            "unnecessary_variable_before_return".to_string(),
            "external_script_as_argument".to_string(),
            "max_function_body_length".to_string(),
            "exit_only_in_main".to_string(),
            "max_positional_params".to_string(),
            "collapsible_if".to_string(),
            "unnecessary_mut".to_string(),
            "unused_helper_functions".to_string(),
        ]),
    }
}

fn documentation_rule_set() -> RuleSet {
    RuleSet {
        name: "documentation".to_string(),
        description: "Documentation quality rules".to_string(),
        rules: HashSet::from(["exported_function_docs".to_string()]),
    }
}

fn performance_rule_set() -> RuleSet {
    RuleSet {
        name: "performance".to_string(),
        description: "Performance optimization hints".to_string(),
        rules: HashSet::from([
            "prefer_nushell_over_jq".to_string(),
            "unused_output".to_string(),
        ]),
    }
}

fn recommended_rule_set() -> RuleSet {
    RuleSet {
        name: "recommended".to_string(),
        description: "Recommended set of rules for most Nushell projects".to_string(),
        rules: HashSet::from([
            "snake_case_variables".to_string(),
            "kebab_case_commands".to_string(),
            "screaming_snake_constants".to_string(),
            "prefer_builtin_ls".to_string(),
            "prefer_builtin_grep".to_string(),
            "prefer_builtin_find".to_string(),
            "prefer_builtin_cat".to_string(),
            "prefer_error_make_for_stderr".to_string(),
            "print_exit_use_error_make".to_string(),
            "escape_string_interpolation_operators".to_string(),
            "exit_only_in_main".to_string(),
            "never_use_echo".to_string(),
            "remove_redundant_in".to_string(),
            "prefer_where_over_each_if".to_string(),
            "unnecessary_ignore".to_string(),
            "pipe_spacing".to_string(),
            "brace_spacing".to_string(),
            "no_trailing_spaces".to_string(),
            "missing_type_annotation".to_string(),
            "typed_pipeline_io".to_string(),
        ]),
    }
}

fn strict_rule_set() -> RuleSet {
    RuleSet {
        name: "strict".to_string(),
        description: "Strict ruleset with all recommended rules at deny level".to_string(),
        rules: HashSet::from([
            "snake_case_variables".to_string(),
            "kebab_case_commands".to_string(),
            "screaming_snake_constants".to_string(),
            "prefer_builtin_ls".to_string(),
            "prefer_builtin_grep".to_string(),
            "prefer_builtin_find".to_string(),
            "prefer_builtin_cat".to_string(),
            "prefer_error_make_for_stderr".to_string(),
            "print_exit_use_error_make".to_string(),
            "escape_string_interpolation_operators".to_string(),
            "exit_only_in_main".to_string(),
            "never_use_echo".to_string(),
            "remove_redundant_in".to_string(),
            "prefer_where_over_each_if".to_string(),
            "unnecessary_ignore".to_string(),
            "pipe_spacing".to_string(),
            "brace_spacing".to_string(),
            "no_trailing_spaces".to_string(),
            "missing_type_annotation".to_string(),
            "typed_pipeline_io".to_string(),
            "forbid_excessive_nesting".to_string(),
            "max_function_body_length".to_string(),
            "max_positional_params".to_string(),
        ]),
    }
}

fn systemd_rule_set() -> RuleSet {
    RuleSet {
        name: "systemd".to_string(),
        description: "Rules for systemd service scripts".to_string(),
        rules: HashSet::from([
            "systemd_journal_prefix".to_string(),
            "prefer_error_make_for_stderr".to_string(),
            "pipeline_handle_errors".to_string(),
        ]),
    }
}

pub static BUILTIN_LINT_SETS: LazyLock<HashMap<&str, RuleSet>> = LazyLock::new(|| {
    HashMap::from([
        ("naming", naming_rule_set()),
        ("idioms", idioms_rule_set()),
        ("pedantic", pedantic_rule_set()),
        ("formatting", formatting_rule_set()),
        ("error-handling", error_handling_rule_set()),
        ("type-safety", type_safety_rule_set()),
        ("code-quality", code_quality_rule_set()),
        ("documentation", documentation_rule_set()),
        ("performance", performance_rule_set()),
        ("recommended", recommended_rule_set()),
        ("strict", strict_rule_set()),
        ("systemd", systemd_rule_set()),
    ])
});

pub static DEFAULT_RULE_MAP: LazyLock<RuleMap> = LazyLock::new(|| RuleMap {
    name: "default".to_string(),
    description: "Default lint levels for all rules".to_string(),
    rules: HashMap::from([
        ("check_complete_exit_code".to_string(), LintLevel::Warn),
        ("collapsible_if".to_string(), LintLevel::Warn),
        ("dangerous_file_operations".to_string(), LintLevel::Warn),
        ("descriptive_error_messages".to_string(), LintLevel::Warn),
        ("error_make_metadata".to_string(), LintLevel::Warn),
        (
            "escape_string_interpolation_operators".to_string(),
            LintLevel::Warn,
        ),
        ("exit_only_in_main".to_string(), LintLevel::Warn),
        ("unnecessary_ignore".to_string(), LintLevel::Warn),
        ("exported_function_docs".to_string(), LintLevel::Warn),
        ("main_positional_args_docs".to_string(), LintLevel::Warn),
        ("main_named_args_docs".to_string(), LintLevel::Warn),
        ("forbid_excessive_nesting".to_string(), LintLevel::Warn),
        ("external_script_as_argument".to_string(), LintLevel::Warn),
        ("inline_single_use_function".to_string(), LintLevel::Warn),
        ("mixed_io_types".to_string(), LintLevel::Warn),
        ("print_and_return_data".to_string(), LintLevel::Warn),
        ("pure_before_side_effects".to_string(), LintLevel::Warn),
        ("kebab_case_commands".to_string(), LintLevel::Warn),
        ("max_function_body_length".to_string(), LintLevel::Warn),
        ("max_positional_params".to_string(), LintLevel::Warn),
        ("missing_type_annotation".to_string(), LintLevel::Warn),
        ("typed_pipeline_io".to_string(), LintLevel::Warn),
        ("prefer_multiline_functions".to_string(), LintLevel::Warn),
        ("prefer_multiline_lists".to_string(), LintLevel::Warn),
        ("prefer_multiline_records".to_string(), LintLevel::Warn),
        ("never_use_echo".to_string(), LintLevel::Warn),
        ("no_trailing_spaces".to_string(), LintLevel::Warn),
        ("nu_parse_error".to_string(), LintLevel::Deny),
        ("omit_list_commas".to_string(), LintLevel::Warn),
        ("pipeline_handle_errors".to_string(), LintLevel::Warn),
        ("prefer_builtin_cat".to_string(), LintLevel::Warn),
        ("prefer_builtin_find".to_string(), LintLevel::Warn),
        ("prefer_builtin_grep".to_string(), LintLevel::Warn),
        ("prefer_builtin_head".to_string(), LintLevel::Warn),
        ("prefer_nushell_over_jq".to_string(), LintLevel::Warn),
        ("prefer_builtin_http".to_string(), LintLevel::Warn),
        ("prefer_builtin_ls".to_string(), LintLevel::Warn),
        ("prefer_builtin_other".to_string(), LintLevel::Warn),
        ("prefer_builtin_sed".to_string(), LintLevel::Warn),
        ("prefer_builtin_sort".to_string(), LintLevel::Warn),
        ("prefer_builtin_tail".to_string(), LintLevel::Warn),
        ("prefer_builtin_uniq".to_string(), LintLevel::Warn),
        ("prefer_compound_assignment".to_string(), LintLevel::Warn),
        ("prefer_direct_use".to_string(), LintLevel::Warn),
        ("prefer_error_make_for_stderr".to_string(), LintLevel::Warn),
        ("print_exit_use_error_make".to_string(), LintLevel::Warn),
        ("prefer_is_not_empty".to_string(), LintLevel::Warn),
        ("prefer_lines_over_split".to_string(), LintLevel::Warn),
        ("prefer_match_over_if_chain".to_string(), LintLevel::Warn),
        ("prefer_parse_command".to_string(), LintLevel::Warn),
        ("prefer_parse_over_each_split".to_string(), LintLevel::Warn),
        ("prefer_path_type".to_string(), LintLevel::Warn),
        ("prefer_pipeline_input".to_string(), LintLevel::Warn),
        ("prefer_range_iteration".to_string(), LintLevel::Warn),
        ("prefer_where_over_each_if".to_string(), LintLevel::Warn),
        ("prefer_where_over_for_if".to_string(), LintLevel::Warn),
        ("remove_redundant_in".to_string(), LintLevel::Warn),
        ("screaming_snake_constants".to_string(), LintLevel::Warn),
        ("snake_case_variables".to_string(), LintLevel::Warn),
        ("brace_spacing".to_string(), LintLevel::Warn),
        ("pipe_spacing".to_string(), LintLevel::Warn),
        ("systemd_journal_prefix".to_string(), LintLevel::Warn),
        ("unnecessary_mut".to_string(), LintLevel::Warn),
        (
            "unnecessary_variable_before_return".to_string(),
            LintLevel::Warn,
        ),
        ("unused_helper_functions".to_string(), LintLevel::Warn),
        ("unused_output".to_string(), LintLevel::Warn),
    ]),
});

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
