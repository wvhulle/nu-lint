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
    pub explanation: String,
    pub rules: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleMap {
    pub name: String,
    pub explanation: String,
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
        explanation: "Linting rules for naming conventions".to_string(),
        rules: HashSet::from([
            "kebab_case_commands".to_string(),
            "screaming_snake_constants".to_string(),
            "snake_case_variables".to_string(),
        ]),
    }
}

fn formatting_rule_set() -> RuleSet {
    RuleSet {
        name: "formatting".to_string(),
        explanation: "Check that code is formatted according to the official Nushell guidelines."
            .to_string(),
        rules: HashSet::from([
            "brace_spacing".to_string(),
            "no_trailing_spaces".to_string(),
            "omit_list_commas".to_string(),
            "pipe_spacing".to_string(),
            "prefer_multiline_functions".to_string(),
            "prefer_multiline_lists".to_string(),
            "prefer_multiline_records".to_string(),
        ]),
    }
}

fn error_handling_rule_set() -> RuleSet {
    RuleSet {
        name: "error-handling".to_string(),
        explanation: "Error handling best practices".to_string(),
        rules: HashSet::from([
            "add_metadata_to_error".to_string(),
            "check_complete_exit_code".to_string(),
            "descriptive_error_messages".to_string(),
            "escape_string_interpolation_operators".to_string(),
            "prefer_complete_for_external_commands".to_string(),
            "prefer_error_make_for_stderr".to_string(),
            "print_exit_use_error_make".to_string(),
        ]),
    }
}

fn type_safety_rule_set() -> RuleSet {
    RuleSet {
        name: "type-safety".to_string(),
        explanation: "Enforce explicit typing of variables and pipelines.".to_string(),
        rules: HashSet::from([
            "external_script_as_argument".to_string(),
            "missing_type_annotation".to_string(),
            "prefer_path_type".to_string(),
            "typed_pipeline_io".to_string(),
        ]),
    }
}

fn side_effects_rule_set() -> RuleSet {
    RuleSet {
        name: "side-effects".to_string(),
        explanation: "Side effects (or effects) are things commands do that escape the type \
                      system, but happen often and may cause unexpected behavior."
            .to_string(),
        rules: HashSet::from([
            "mixed_io_types".to_string(),
            "print_and_return_data".to_string(),
            "pure_before_side_effects".to_string(),
        ]),
    }
}

fn documentation_rule_set() -> RuleSet {
    RuleSet {
        name: "documentation".to_string(),
        explanation: "Documentation quality rules".to_string(),
        rules: HashSet::from([
            "exported_function_docs".to_string(),
            "descriptive_error_messages".into(),
            "main_positional_args_docs".to_string(),
            "main_named_args_docs".to_string(),
        ]),
    }
}

fn performance_rule_set() -> RuleSet {
    RuleSet {
        name: "performance".to_string(),
        explanation: "Performance optimization hints".to_string(),
        rules: HashSet::from([
            "prefer_builtin_cat".to_string(),
            "prefer_builtin_echo".to_string(),
            "prefer_builtin_find".to_string(),
            "prefer_builtin_grep".to_string(),
            "prefer_builtin_head".to_string(),
            "prefer_builtin_http".to_string(),
            "prefer_builtin_ls".to_string(),
            "prefer_builtin_sed".to_string(),
            "prefer_builtin_sort".to_string(),
            "prefer_builtin_tail".to_string(),
            "prefer_builtin_uniq".to_string(),
            "prefer_compound_assignment".to_string(),
            "prefer_direct_use".to_string(),
            "prefer_is_not_empty".to_string(),
            "prefer_lines_over_split".to_string(),
            "prefer_nushell_over_jq".to_string(),
            "prefer_parse_over_each_split".to_string(),
            "prefer_parse_over_split_get".to_string(),
            "prefer_pipeline_input".to_string(),
            "prefer_range_iteration".to_string(),
            "prefer_where_over_each_if".to_string(),
            "prefer_where_over_for_if".to_string(),
            "redundant_ignore".to_string(),
            "remove_redundant_in".to_string(),
            "unnecessary_variable_before_return".to_string(),
        ]),
    }
}

fn systemd_rule_set() -> RuleSet {
    RuleSet {
        name: "systemd".to_string(),
        explanation: "Rules for systemd service scripts".to_string(),
        rules: HashSet::from(["systemd_journal_prefix".to_string()]),
    }
}

pub static BUILTIN_LINT_SETS: LazyLock<HashMap<&str, RuleSet>> = LazyLock::new(|| {
    HashMap::from([
        ("documentation", documentation_rule_set()),
        ("error-handling", error_handling_rule_set()),
        ("formatting", formatting_rule_set()),
        ("naming", naming_rule_set()),
        ("performance", performance_rule_set()),
        ("side-effects", side_effects_rule_set()),
        ("systemd", systemd_rule_set()),
        ("type-safety", type_safety_rule_set()),
    ])
});

pub static DEFAULT_RULE_MAP: LazyLock<RuleMap> = LazyLock::new(|| RuleMap {
    name: "default".to_string(),
    explanation: "Default lint levels for all rules".to_string(),
    rules: HashMap::from([
        ("check_complete_exit_code".to_string(), LintLevel::Warn),
        ("collapsible_if".to_string(), LintLevel::Warn),
        ("dangerous_file_operations".to_string(), LintLevel::Warn),
        ("descriptive_error_messages".to_string(), LintLevel::Warn),
        ("add_metadata_to_error".to_string(), LintLevel::Warn),
        (
            "escape_string_interpolation_operators".to_string(),
            LintLevel::Deny,
        ),
        ("exit_only_in_main".to_string(), LintLevel::Deny),
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
        ("no_trailing_spaces".to_string(), LintLevel::Warn),
        ("nu_deprecated".to_string(), LintLevel::Deny),
        ("nu_parse_error".to_string(), LintLevel::Deny),
        ("omit_list_commas".to_string(), LintLevel::Warn),
        (
            "prefer_complete_for_external_commands".to_string(),
            LintLevel::Warn,
        ),
        ("prefer_builtin_cat".to_string(), LintLevel::Warn),
        ("prefer_builtin_echo".to_string(), LintLevel::Warn),
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
        ("prefer_direct_use".to_string(), LintLevel::Deny),
        ("prefer_error_make_for_stderr".to_string(), LintLevel::Warn),
        ("print_exit_use_error_make".to_string(), LintLevel::Warn),
        ("prefer_is_not_empty".to_string(), LintLevel::Warn),
        ("prefer_lines_over_split".to_string(), LintLevel::Warn),
        ("prefer_match_over_if_chain".to_string(), LintLevel::Warn),
        ("prefer_parse_over_split_get".to_string(), LintLevel::Warn),
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
        ("systemd_journal_prefix".to_string(), LintLevel::Allow),
        ("unnecessary_mut".to_string(), LintLevel::Warn),
        (
            "unnecessary_variable_before_return".to_string(),
            LintLevel::Warn,
        ),
        ("unused_helper_functions".to_string(), LintLevel::Warn),
        ("redundant_ignore".to_string(), LintLevel::Warn),
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
