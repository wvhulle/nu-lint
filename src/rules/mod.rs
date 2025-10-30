pub mod avoid_mutable_accumulation;
pub mod brace_spacing;
pub mod check_complete_exit_code;
pub mod completion_function_naming;
pub mod dangerous_file_operations;
pub mod descriptive_error_messages;
pub mod discourage_bare_ignore;
pub mod discourage_underscore_commands;
pub mod escape_string_interpolation_operators;
pub mod exit_only_in_main;
pub mod exported_function_docs;
pub mod kebab_case_commands;
pub mod max_positional_params;
pub mod missing_command_docs;
pub mod missing_type_annotation;
pub mod multiline_formatting;
pub mod no_trailing_spaces;
pub mod nu_parse_error;
pub mod omit_list_commas;
pub mod pipe_spacing;
pub mod pipeline_handle_errors;
pub mod prefer_builtin;
pub mod prefer_compound_assignment;
pub mod prefer_each_over_for;
pub mod prefer_error_make;
pub mod prefer_from_json;
pub mod prefer_is_not_empty;
pub mod prefer_lines_over_split;
pub mod prefer_match_over_if_chain;
pub mod prefer_nushell_data_ops;
pub mod prefer_parse_command;
pub mod prefer_parse_over_each_split;
pub mod prefer_range_iteration;
pub mod prefer_structured_data_flow;
pub mod prefer_where_over_each_if;
pub mod require_external_command_error_check;
pub mod require_long_pipeline_error_handling;
pub mod screaming_snake_constants;
pub mod snake_case_variables;
pub mod systemd_journal_prefix;
pub mod unnecessary_mut;
pub mod unnecessary_variable_before_return;

use std::collections::HashMap;

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

        registry.register(avoid_mutable_accumulation::rule());
        registry.register(brace_spacing::rule());
        registry.register(check_complete_exit_code::rule());
        registry.register(completion_function_naming::rule());
        registry.register(dangerous_file_operations::rule());
        registry.register(descriptive_error_messages::rule());
        registry.register(discourage_bare_ignore::rule());
        registry.register(discourage_underscore_commands::rule());
        registry.register(escape_string_interpolation_operators::rule());
        registry.register(exit_only_in_main::rule());
        registry.register(exported_function_docs::rule());
        registry.register(kebab_case_commands::rule());
        registry.register(max_positional_params::rule());
        registry.register(missing_command_docs::rule());
        registry.register(missing_type_annotation::rule());
        registry.register(multiline_formatting::rule());
        registry.register(no_trailing_spaces::rule());
        registry.register(nu_parse_error::rule());
        registry.register(omit_list_commas::rule());
        registry.register(pipe_spacing::rule());
        // New individual prefer_builtin_* rules
        registry.register(prefer_builtin::prefer_builtin_ls());
        registry.register(prefer_builtin::prefer_builtin_cat());
        registry.register(prefer_builtin::prefer_builtin_grep());
        registry.register(prefer_builtin::prefer_builtin_find());
        registry.register(prefer_builtin::prefer_builtin_head());
        registry.register(prefer_builtin::prefer_builtin_tail());
        registry.register(prefer_builtin::prefer_builtin_sort());
        registry.register(prefer_builtin::prefer_builtin_uniq());
        registry.register(prefer_builtin::prefer_builtin_sed());
        registry.register(prefer_builtin::prefer_builtin_other());
        registry.register(prefer_builtin::prefer_builtin_jq());
        registry.register(prefer_compound_assignment::rule());
        registry.register(prefer_each_over_for::rule());
        registry.register(prefer_error_make::rule());
        registry.register(prefer_from_json::rule());
        registry.register(prefer_is_not_empty::rule());
        registry.register(prefer_lines_over_split::rule());
        registry.register(prefer_match_over_if_chain::rule());
        registry.register(prefer_nushell_data_ops::rule());
        registry.register(prefer_parse_command::rule());
        registry.register(prefer_parse_over_each_split::rule());
        registry.register(prefer_range_iteration::rule());
        registry.register(prefer_structured_data_flow::rule());
        registry.register(prefer_where_over_each_if::rule());
        registry.register(require_external_command_error_check::rule());
        registry.register(require_long_pipeline_error_handling::rule());
        registry.register(screaming_snake_constants::rule());
        registry.register(pipeline_handle_errors::rule());
        registry.register(snake_case_variables::rule());
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
