use core::{
    fmt::{self, Display},
    hash::{Hash, Hasher},
};
use std::{collections::HashMap, sync::LazyLock};

use serde::Serialize;

use crate::LintLevel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintSet {
    pub name: String,
    pub description: String,
    pub rules: HashMap<String, LintLevel>,
}

impl Serialize for LintSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Just serialize the name of the lint set
        serializer.serialize_str(&self.name)
    }
}

static BUILTIN_LINT_SETS: LazyLock<HashMap<&str, LintSet>> = LazyLock::new(|| {
    HashMap::from([
        (
            "naming",
            LintSet {
                name: "naming".to_string(),
                description: "Linting rules for naming conventions".to_string(),
                rules: HashMap::from([
                    ("snake_case_variables".to_string(), LintLevel::Warn),
                    ("kebab_case_commands".to_string(), LintLevel::Warn),
                    ("screaming_snake_constants".to_string(), LintLevel::Warn),
                ]),
            },
        ),
        (
            "idioms",
            LintSet {
                name: "idioms".to_string(),
                description: "Linting rules for idiomatic Nushell expressions".to_string(),
                rules: HashMap::from([
                    ("prefer_builtin_ls".to_string(), LintLevel::Warn),
                    ("prefer_parse_command".to_string(), LintLevel::Warn),
                    ("prefer_where_over_each_if".to_string(), LintLevel::Warn),
                    ("prefer_builtin_uniq".to_string(), LintLevel::Warn),
                    ("prefer_builtin_http".to_string(), LintLevel::Warn),
                    ("prefer_builtin_grep".to_string(), LintLevel::Warn),
                    ("remove_redundant_in".to_string(), LintLevel::Warn),
                    ("prefer_builtin_find".to_string(), LintLevel::Warn),
                    ("prefer_lines_over_split".to_string(), LintLevel::Warn),
                    ("prefer_builtin_cat".to_string(), LintLevel::Warn),
                    ("prefer_compound_assignment".to_string(), LintLevel::Warn),
                    ("unnecessary_ignore".to_string(), LintLevel::Warn),
                    ("prefer_parse_over_each_split".to_string(), LintLevel::Warn),
                    ("prefer_builtin_sed".to_string(), LintLevel::Warn),
                    ("prefer_match_over_if_chain".to_string(), LintLevel::Warn),
                    ("prefer_where_over_for_if".to_string(), LintLevel::Warn),
                    ("prefer_builtin_head".to_string(), LintLevel::Warn),
                    ("prefer_pipeline_input".to_string(), LintLevel::Warn),
                    ("prefer_builtin_sort".to_string(), LintLevel::Warn),
                    ("prefer_builtin_tail".to_string(), LintLevel::Warn),
                    ("prefer_builtin_other".to_string(), LintLevel::Warn),
                    ("never_use_echo".to_string(), LintLevel::Deny),
                    ("prefer_is_not_empty".to_string(), LintLevel::Warn),
                    ("prefer_range_iteration".to_string(), LintLevel::Warn),
                ]),
            },
        ),
        (
            "pedantic",
            LintSet {
                name: "pedantic".to_string(),
                description: "Strict linting for high code quality standards".to_string(),
                rules: HashMap::from([
                    ("forbid_excessive_nesting".to_string(), LintLevel::Deny),
                    ("max_function_body_length".to_string(), LintLevel::Warn),
                    ("max_positional_params".to_string(), LintLevel::Warn),
                    ("inline_single_use_function".to_string(), LintLevel::Warn),
                    (
                        "unnecessary_variable_before_return".to_string(),
                        LintLevel::Warn,
                    ),
                    ("collapsible_if".to_string(), LintLevel::Warn),
                    ("unnecessary_mut".to_string(), LintLevel::Warn),
                    ("unused_helper_functions".to_string(), LintLevel::Warn),
                    ("prefer_direct_use".to_string(), LintLevel::Warn),
                ]),
            },
        ),
        (
            "formatting",
            LintSet {
                name: "formatting".to_string(),
                description: "Code formatting and style rules".to_string(),
                rules: HashMap::from([
                    ("no_trailing_spaces".to_string(), LintLevel::Warn),
                    ("prefer_multiline_lists".to_string(), LintLevel::Allow),
                    ("pipe_spacing".to_string(), LintLevel::Warn),
                    ("prefer_multiline_records".to_string(), LintLevel::Allow),
                    ("omit_list_commas".to_string(), LintLevel::Warn),
                    ("prefer_multiline_functions".to_string(), LintLevel::Allow),
                    ("brace_spacing".to_string(), LintLevel::Warn),
                ]),
            },
        ),
        (
            "error-handling",
            LintSet {
                name: "error-handling".to_string(),
                description: "Error handling best practices".to_string(),
                rules: HashMap::from([
                    ("prefer_error_make_for_stderr".to_string(), LintLevel::Warn),
                    ("descriptive_error_messages".to_string(), LintLevel::Warn),
                    ("print_exit_use_error_make".to_string(), LintLevel::Warn),
                    (
                        "escape_string_interpolation_operators".to_string(),
                        LintLevel::Deny,
                    ),
                    ("pipeline_handle_errors".to_string(), LintLevel::Warn),
                    ("check_complete_exit_code".to_string(), LintLevel::Warn),
                    ("error_make_metadata".to_string(), LintLevel::Warn),
                    ("dangerous_file_operations".to_string(), LintLevel::Warn),
                ]),
            },
        ),
        (
            "type-safety",
            LintSet {
                name: "type-safety".to_string(),
                description: "Type annotation and safety rules".to_string(),
                rules: HashMap::from([
                    ("missing_type_annotation".to_string(), LintLevel::Warn),
                    ("prefer_path_type".to_string(), LintLevel::Warn),
                    ("typed_pipeline_io".to_string(), LintLevel::Warn),
                ]),
            },
        ),
        (
            "code-quality",
            LintSet {
                name: "code-quality".to_string(),
                description: "General code quality and maintainability rules".to_string(),
                rules: HashMap::from([
                    ("forbid_excessive_nesting".to_string(), LintLevel::Warn),
                    ("prefer_direct_use".to_string(), LintLevel::Warn),
                    ("inline_single_use_function".to_string(), LintLevel::Allow),
                    (
                        "unnecessary_variable_before_return".to_string(),
                        LintLevel::Warn,
                    ),
                    ("external_script_as_argument".to_string(), LintLevel::Warn),
                    ("max_function_body_length".to_string(), LintLevel::Warn),
                    ("nu_parse_error".to_string(), LintLevel::Deny),
                    ("exit_only_in_main".to_string(), LintLevel::Deny),
                    ("max_positional_params".to_string(), LintLevel::Warn),
                    ("collapsible_if".to_string(), LintLevel::Warn),
                    ("unnecessary_mut".to_string(), LintLevel::Warn),
                    ("unused_helper_functions".to_string(), LintLevel::Warn),
                ]),
            },
        ),
        (
            "documentation",
            LintSet {
                name: "documentation".to_string(),
                description: "Documentation quality rules".to_string(),
                rules: HashMap::from([("exported_function_docs".to_string(), LintLevel::Warn)]),
            },
        ),
        (
            "performance",
            LintSet {
                name: "performance".to_string(),
                description: "Performance optimization hints".to_string(),
                rules: HashMap::from([
                    ("prefer_nushell_over_jq".to_string(), LintLevel::Warn),
                    ("unused_output".to_string(), LintLevel::Warn),
                ]),
            },
        ),
        (
            "recommended",
            LintSet {
                name: "recommended".to_string(),
                description: "Recommended set of rules for most Nushell projects".to_string(),
                rules: HashMap::from([
                    ("snake_case_variables".to_string(), LintLevel::Warn),
                    ("kebab_case_commands".to_string(), LintLevel::Warn),
                    ("screaming_snake_constants".to_string(), LintLevel::Warn),
                    ("prefer_builtin_ls".to_string(), LintLevel::Warn),
                    ("prefer_builtin_grep".to_string(), LintLevel::Warn),
                    ("prefer_builtin_find".to_string(), LintLevel::Warn),
                    ("prefer_builtin_cat".to_string(), LintLevel::Warn),
                    ("prefer_error_make_for_stderr".to_string(), LintLevel::Warn),
                    ("print_exit_use_error_make".to_string(), LintLevel::Warn),
                    (
                        "escape_string_interpolation_operators".to_string(),
                        LintLevel::Deny,
                    ),
                    ("nu_parse_error".to_string(), LintLevel::Deny),
                    ("exit_only_in_main".to_string(), LintLevel::Deny),
                    ("never_use_echo".to_string(), LintLevel::Deny),
                    ("remove_redundant_in".to_string(), LintLevel::Warn),
                    ("prefer_where_over_each_if".to_string(), LintLevel::Warn),
                    ("unnecessary_ignore".to_string(), LintLevel::Warn),
                    ("pipe_spacing".to_string(), LintLevel::Warn),
                    ("brace_spacing".to_string(), LintLevel::Warn),
                    ("no_trailing_spaces".to_string(), LintLevel::Warn),
                    ("missing_type_annotation".to_string(), LintLevel::Warn),
                    ("typed_pipeline_io".to_string(), LintLevel::Warn),
                ]),
            },
        ),
        (
            "strict",
            LintSet {
                name: "strict".to_string(),
                description: "Strict ruleset with all recommended rules at deny level".to_string(),
                rules: HashMap::from([
                    ("snake_case_variables".to_string(), LintLevel::Deny),
                    ("kebab_case_commands".to_string(), LintLevel::Deny),
                    ("screaming_snake_constants".to_string(), LintLevel::Deny),
                    ("prefer_builtin_ls".to_string(), LintLevel::Deny),
                    ("prefer_builtin_grep".to_string(), LintLevel::Deny),
                    ("prefer_builtin_find".to_string(), LintLevel::Deny),
                    ("prefer_builtin_cat".to_string(), LintLevel::Deny),
                    ("prefer_error_make_for_stderr".to_string(), LintLevel::Deny),
                    ("print_exit_use_error_make".to_string(), LintLevel::Deny),
                    (
                        "escape_string_interpolation_operators".to_string(),
                        LintLevel::Deny,
                    ),
                    ("nu_parse_error".to_string(), LintLevel::Deny),
                    ("exit_only_in_main".to_string(), LintLevel::Deny),
                    ("never_use_echo".to_string(), LintLevel::Deny),
                    ("remove_redundant_in".to_string(), LintLevel::Deny),
                    ("prefer_where_over_each_if".to_string(), LintLevel::Deny),
                    ("unnecessary_ignore".to_string(), LintLevel::Deny),
                    ("pipe_spacing".to_string(), LintLevel::Deny),
                    ("brace_spacing".to_string(), LintLevel::Deny),
                    ("no_trailing_spaces".to_string(), LintLevel::Deny),
                    ("missing_type_annotation".to_string(), LintLevel::Deny),
                    ("typed_pipeline_io".to_string(), LintLevel::Deny),
                    ("forbid_excessive_nesting".to_string(), LintLevel::Deny),
                    ("max_function_body_length".to_string(), LintLevel::Deny),
                    ("max_positional_params".to_string(), LintLevel::Deny),
                ]),
            },
        ),
        (
            "systemd",
            LintSet {
                name: "systemd".to_string(),
                description: "Rules for systemd service scripts".to_string(),
                rules: HashMap::from([
                    ("systemd_journal_prefix".to_string(), LintLevel::Warn),
                    ("prefer_error_make_for_stderr".to_string(), LintLevel::Deny),
                    ("pipeline_handle_errors".to_string(), LintLevel::Deny),
                ]),
            },
        ),
    ])
});

#[must_use]
pub fn builtin_lint_sets() -> &'static HashMap<&'static str, LintSet> {
    &BUILTIN_LINT_SETS
}

impl Hash for LintSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Display for LintSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
