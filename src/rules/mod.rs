pub mod check_complete_exit_code;
pub mod collapsible_if;
pub mod dangerous_file_operations;
pub mod descriptive_error_messages;
pub mod error_suppression_over_ignore;
pub mod escape_string_interpolation_operators;
pub mod exit_only_in_main;
pub mod exported_function_docs;
pub mod external_script_as_argument;
pub mod forbid_excessive_nesting;

pub mod max_positional_params;
pub mod missing_type_annotation;
pub mod naming;

pub mod nu_parse_error;

pub mod pipeline_handle_errors;

pub mod prefer_compound_assignment;
pub mod prefer_direct_use;
pub mod prefer_error_make;
pub mod prefer_is_not_empty;
pub mod prefer_lines_over_split;
pub mod prefer_match_over_if_chain;
pub mod prefer_parse_command;
pub mod prefer_parse_over_each_split;
pub mod prefer_pipeline_input;
pub mod prefer_range_iteration;
pub mod prefer_where_over_each_if;
pub mod prefer_where_over_for_if;
pub mod remove_redundant_in;

pub mod replace_by_builtin;
pub mod spacing;
pub mod systemd_journal_prefix;
pub mod unnecessary_mut;
pub mod unnecessary_variable_before_return;
use std::collections::HashMap;

use naming::{
    completion_function_naming, kebab_case_commands, screaming_snake_constants,
    snake_case_variables,
};
use spacing::{multiline_formatting, no_trailing_spaces, omit_list_commas};

use crate::rule::Rule;

pub struct RuleRegistry {
    rules: HashMap<&'static str, Rule>,
}

impl RuleRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    pub fn register(&mut self, rule: Rule) {
        self.rules.insert(rule.id, rule);
    }

    #[must_use]
    pub fn get_rule(&self, id: &str) -> Option<&Rule> {
        self.rules.get(id)
    }

    pub fn all_rules(&self) -> impl Iterator<Item = &Rule> {
        self.rules.values()
    }

    #[must_use]
    pub fn with_default_rules() -> Self {
        let mut registry = Self::new();
        // TODO: add rule that detects custom commands with a body (apart from comments)
        // of length 1, used just once and suggests inlining at call-site.
        registry.register(check_complete_exit_code::rule());
        registry.register(collapsible_if::rule());
        registry.register(completion_function_naming::rule());
        registry.register(dangerous_file_operations::rule());
        registry.register(descriptive_error_messages::rule());
        registry.register(error_suppression_over_ignore::rule());
        registry.register(escape_string_interpolation_operators::rule());
        registry.register(exit_only_in_main::rule());
        registry.register(exported_function_docs::rule());
        registry.register(forbid_excessive_nesting::rule());
        registry.register(external_script_as_argument::rule());
        registry.register(kebab_case_commands::rule());
        registry.register(max_positional_params::rule());
        registry.register(missing_type_annotation::rule());
        registry.register(multiline_formatting::rule());
        registry.register(replace_by_builtin::echo::rule());
        registry.register(no_trailing_spaces::rule());
        registry.register(nu_parse_error::rule());
        registry.register(omit_list_commas::rule());
        registry.register(pipeline_handle_errors::rule());
        registry.register(replace_by_builtin::cat::rule());
        registry.register(replace_by_builtin::find::rule());
        registry.register(replace_by_builtin::grep::rule());
        registry.register(replace_by_builtin::head::rule());
        registry.register(replace_by_builtin::jq::rule());
        registry.register(replace_by_builtin::ls::rule());
        registry.register(replace_by_builtin::other::rule());
        registry.register(replace_by_builtin::sed::rule());
        registry.register(replace_by_builtin::sort::rule());
        registry.register(replace_by_builtin::tail::rule());
        registry.register(replace_by_builtin::uniq::rule());
        registry.register(prefer_compound_assignment::rule());
        registry.register(prefer_direct_use::rule());
        registry.register(prefer_error_make::rule());
        registry.register(prefer_is_not_empty::rule());
        registry.register(prefer_lines_over_split::rule());
        registry.register(prefer_match_over_if_chain::rule());
        registry.register(prefer_parse_command::rule());
        registry.register(prefer_parse_over_each_split::rule());
        registry.register(prefer_pipeline_input::rule());
        registry.register(prefer_range_iteration::rule());
        registry.register(prefer_where_over_each_if::rule());
        registry.register(prefer_where_over_for_if::rule());
        registry.register(remove_redundant_in::rule());
        registry.register(screaming_snake_constants::rule());
        registry.register(snake_case_variables::rule());
        registry.register(spacing::brace_spacing::rule());
        registry.register(spacing::pipe_spacing::rule());
        registry.register(systemd_journal_prefix::rule());
        registry.register(unnecessary_mut::rule());
        registry.register(unnecessary_variable_before_return::rule());

        registry
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}
