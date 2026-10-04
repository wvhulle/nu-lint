use core::fmt::{self, Display};

use crate::rule::Rule;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub name: &'static str,
    pub description: &'static str,
    pub rules: &'static [&'static dyn Rule],
}

const ERROR_HANDLING: Group = Group {
    name: "runtime-errors",
    description: "Preventing unexpected runtime behaviour.",
    rules: &[
        super::add_hat_external_commands::RULE,
        super::fragile_last_exit_code::RULE,
        super::check_complete_exit_code::RULE,
        super::serialize_data_for_external::RULE,
        super::documentation::descriptive_error_messages::RULE,
        super::unescaped_interpolation::RULE,
        super::exit_only_in_main::RULE,
        super::check_typed_flag_before_use::RULE,
        super::non_final_failure_check::RULE,
        super::error_make::error_make_for_non_fatal::RULE,
        super::try_instead_of_do::RULE,
        super::unsafe_dynamic_record_access::RULE,
        super::missing_stdin_in_shebang::RULE,
        super::dynamic_script_import::RULE,
        super::catch_builtin_error_try::RULE,
        super::default_empty_string_masks_missing::RULE,
        super::unchecked_cell_path_index::RULE,
        super::unchecked_get_index::RULE,
        super::unhandled_external_error::RULE,
        super::source_to_use::RULE,
        super::spread_list_to_external::RULE,
        super::glob_may_drop_quotes::RULE,
        super::require_main_with_stdin::RULE,
    ],
};

const TYPE_SAFETY: Group = Group {
    name: "type-safety",
    description: "Annotate with type hints where possible.",
    rules: &[
        super::external_script_as_argument::RULE,
        super::nothing_outside_signature::RULE,
        super::typing::add_type_hints_arguments::RULE,
        super::filesystem::string_param_as_path::RULE,
        super::typing::missing_output_type::RULE,
        super::typing::missing_in_type::RULE,
        super::redundant_nu_subprocess::RULE,
        super::dynamic_script_import::RULE,
    ],
};

const IDIOMATIC: Group = Group {
    name: "idioms",
    description: "Simplifications unique to the Nu language.",
    rules: &[
        super::not_is_empty_to_is_not_empty::RULE,
        super::columns_in_to_has::RULE,
        super::columns_not_in_to_not_has::RULE,
        super::dispatch_with_subcommands::RULE,
        super::get_optional_to_has::RULE,
        super::get_optional_to_not_has::RULE,
        super::hardcoded_math_constants::RULE,
        super::transpose_items::RULE,
        super::merge_get_cell_path::RULE,
        super::merge_multiline_print::RULE,
        super::positional_to_pipeline::RULE,
        super::source_to_use::RULE,
        super::compound_assignment::RULE,
        super::contains_to_regex_op::RULE,
        super::ansi_over_escape_codes::RULE,
        super::append_to_concat_assign::RULE,
        super::custom_log_command::RULE,
        super::chained_append::RULE,
        super::record_assignments::USE_RECORD_SPREAD,
        super::record_assignments::USE_LOAD_ENV,
        super::remove_hat_not_builtin::RULE,
        super::division_to_format_duration::RULE,
        super::ignore_over_dev_null::RULE,
        super::redundant_echo::RULE,
    ],
};

const PARSING: Group = Group {
    name: "parsing",
    description: "Better ways to parse and transform text data.",
    rules: &[
        super::parsing::lines_instead_of_split::RULE,
        super::never_space_split::RULE,
        super::parsing::lines_each_to_parse::RULE,
        super::parsing::simplify_regex_parse::RULE,
        super::parsing::split_row_get_multistatement::RULE,
        super::parsing::split_first_to_parse::RULE,
        super::parsing::split_row_get_inline::RULE,
        super::parsing::split_row_space_to_split_words::RULE,
    ],
};

const FILESYSTEM: Group = Group {
    name: "filesystem",
    description: "Simplify file and path operations.",
    rules: &[
        super::filesystem::from_after_parsed_open::RULE,
        super::filesystem::open_raw_from_to_open::RULE,
        super::filesystem::string_param_as_path::RULE,
    ],
};

const FILTERING: Group = Group {
    name: "filtering",
    description: "Better patterns for filtering and selecting data.",
    rules: &[
        super::filtering::each_if_to_where::RULE,
        super::filtering::for_filter_to_where::RULE,
        super::filtering::omit_it_in_row_condition::RULE,
        super::filtering::slice_to_drop::RULE,
        super::filtering::slice_to_last::RULE,
        super::filtering::slice_to_skip::RULE,
        super::filtering::slice_to_take::RULE,
        super::filtering::uniq_to_set_operation::RULE,
        super::filtering::where_closure_drop_parameter::RULE,
        super::remove_redundant_in::RULE,
    ],
};

const ITERATION: Group = Group {
    name: "iteration",
    description: "Better patterns for loops and iteration.",
    rules: &[
        super::range_for_iteration::loop_counter::RULE,
        super::range_for_iteration::while_counter::RULE,
    ],
};

const DEAD_CODE: Group = Group {
    name: "dead-code",
    description: "Remove unused or redundant code",
    rules: &[
        super::self_import::RULE,
        super::unnecessary_accumulate::RULE,
        super::assign_then_return::RULE,
        super::do_not_compare_booleans::RULE,
        super::if_null_to_default::RULE,
        super::redundant_ignore::RULE,
        super::unnecessary_mut::RULE,
        super::unused_helper_functions::RULE,
        super::unused_parameter::RULE,
        super::unused_variable::RULE,
        super::script_export_main::RULE,
        super::string_may_be_bare::RULE,
        super::single_call_command::RULE,
        super::append_to_concat_assign::RULE,
    ],
};

const PERFORMANCE: Group = Group {
    name: "performance",
    description: "Rules with potential performance impact",
    rules: &[
        super::redundant_nu_subprocess::RULE,
        super::dispatch_with_subcommands::RULE,
        super::self_import::RULE,
        super::positional_to_pipeline::RULE,
        super::unnecessary_accumulate::RULE,
        super::merge_multiline_print::RULE,
        super::chained_str_transform::RULE,
        super::streaming_hidden_by_complete::RULE,
        super::chained_append::RULE,
    ],
};

const DOCUMENTATION: Group = Group {
    name: "documentation",
    description: "Improve usefullness user-facing messages.",
    rules: &[
        super::documentation::add_doc_comment_exported_fn::RULE,
        super::documentation::descriptive_error_messages::RULE,
        super::error_make::add_label_to_error::RULE,
        super::error_make::add_help_to_error::RULE,
        super::error_make::add_span_to_label::RULE,
        super::error_make::add_url_to_error::RULE,
        super::documentation::main_positional_args_docs::RULE,
        super::documentation::main_named_args_docs::RULE,
        super::max_positional_params::RULE,
        super::explicit_long_flags::RULE,
        super::list_param_to_variadic::RULE,
    ],
};

const EXTERNAL_TOOLS: Group = Group {
    name: "external",
    description: "Replace external commands with Nu built-ins where the result is equivalent.",
    rules: &[
        super::external_tools::cat_to_open::RULE,
        super::external_tools::curl_wget_to_http::RULE,
        super::external_tools::cd_to_builtin::RULE,
        super::external_tools::date_to_builtin::RULE,
        super::external_tools::find_to_glob::RULE,
        super::external_tools::grep_to_where::RULE,
        super::external_tools::head_tail_to_first_last::RULE,
        super::external_tools::jq_to_nu_pipeline::RULE,
        super::external_tools::ls_to_builtin::RULE,
        super::external_tools::read_to_input::RULE,
        super::external_tools::sed_to_str_replace::RULE,
        super::external_tools::sort_to_builtin::RULE,
        super::external_tools::system_info_to_sys::RULE,
        super::external_tools::tac_to_reverse::RULE,
        super::external_tools::wc_to_length::RULE,
    ],
};

const FORMATTING: Group = Group {
    name: "formatting",
    description: "Formatting according to style-guide.",
    rules: &[
        super::ansi_over_escape_codes::RULE,
        super::collapsible_if::RULE,
        super::forbid_excessive_nesting::RULE,
        super::max_function_body_length::RULE,
        super::if_else_chain_to_match::RULE,
        super::spacing::block_brace_spacing::RULE,
        super::spacing::closure_brace_pipe_spacing::RULE,
        super::spacing::closure_pipe_body_spacing::RULE,
        super::spacing::no_trailing_spaces::RULE,
        super::spacing::omit_list_commas::RULE,
        super::spacing::pipe_spacing::RULE,
        super::spacing::record_brace_spacing::RULE,
        super::spacing::reflow_wide_pipelines::RULE,
        super::spacing::reflow_wide_lists::RULE,
        super::spacing::wrap_wide_records::RULE,
    ],
};

const NAMING: Group = Group {
    name: "naming",
    description: "Follow official naming conventions",
    rules: &[
        super::naming::kebab_case_commands::RULE,
        super::naming::screaming_snake_constants::RULE,
        super::naming::snake_case_variables::RULE,
        super::error_make::add_label_to_error::RULE,
    ],
};

const SIDE_EFFECTS: Group = Group {
    name: "effects",
    description: "Handle built-in and external commands with side-effects.",
    rules: &[
        super::dangerous_file_operations::RULE,
        super::errors_to_stderr::RULE,
        super::side_effects::dont_mix_different_effects::RULE,
        super::side_effects::print_and_return_data::RULE,
        super::side_effects::each_nothing_to_for_loop::RULE,
        super::side_effects::silence_stderr_data::RULE,
    ],
};

const UPSTREAM: Group = Group {
    name: "upstream",
    description: "Forward warnings and errors of the Nu parser.",
    rules: &[
        super::dynamic_script_import::RULE,
        super::upstream::nu_deprecated::RULE,
        super::upstream::nu_parse_error::RULE,
    ],
};

pub const ALL_GROUPS: &[Group] = &[
    IDIOMATIC,
    PARSING,
    FILESYSTEM,
    DEAD_CODE,
    ITERATION,
    ERROR_HANDLING,
    FILTERING,
    PERFORMANCE,
    TYPE_SAFETY,
    DOCUMENTATION,
    SIDE_EFFECTS,
    EXTERNAL_TOOLS,
    FORMATTING,
    NAMING,
    UPSTREAM,
];

/// Find all groups that contain the given `rule_id`
pub fn groups_for_rule(rule_id: &str) -> Vec<&'static str> {
    ALL_GROUPS
        .iter()
        .filter(|g| g.rules.iter().any(|r| r.id() == rule_id))
        .map(|g| g.name)
        .collect()
}

impl Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Write, fs};

    use super::{ALL_GROUPS, Group};

    fn readme_markdown() -> String {
        ALL_GROUPS.iter().map(group_details_markdown).collect()
    }

    fn group_details_markdown(group: &Group) -> String {
        let rule_lines = group.rules.iter().fold(String::new(), |mut lines, rule| {
            let auto_fix_suffix = if rule.has_auto_fix() {
                " (auto-fix)"
            } else {
                ""
            };
            writeln!(
                lines,
                "- `{}`{auto_fix_suffix}: {}",
                rule.id(),
                rule.short_description()
            )
            .unwrap();
            lines
        });
        format!(
            "<details>\n<summary><code>{}</code> ({} rules): \
             {}</summary>\n\n{rule_lines}\n</details>\n\n",
            group.name,
            group.rules.len(),
            group.description
        )
    }

    const README_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/README.md");
    const START_MARKER: &str = "<!-- start-rule-groups -->\n";
    const END_MARKER: &str = "<!-- end-rule-groups -->";

    #[test]
    fn readme_rule_list_is_up_to_date() {
        let readme = fs::read_to_string(README_PATH).unwrap();
        let (before_list, rest) = readme.split_once(START_MARKER).unwrap();
        let (_, after_list) = rest.split_once(END_MARKER).unwrap();
        let regenerated = format!(
            "{before_list}{START_MARKER}{}{END_MARKER}{after_list}",
            readme_markdown()
        );
        if regenerated != readme {
            fs::write(README_PATH, regenerated).unwrap();
            panic!("README.md rule list was outdated and has been regenerated; commit the change");
        }
    }
}
