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
        super::unchecked_cell_path_index::RULE,
        super::unchecked_get_index::RULE,
        super::wrap_external_with_complete::RULE,
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
        super::record_assignments::MERGE_FLAT_UPSERT,
        super::record_assignments::MERGE_NESTED_UPSERT,
        super::record_assignments::USE_LOAD_ENV,
        super::remove_hat_not_builtin::RULE,
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

const POSIX_TOOLS: Group = Group {
    name: "posix",
    description: "Replace common bash/POSIX patterns.",
    rules: &[
        super::ignore_over_dev_null::RULE,
        super::posix_tools::awk_to_pipeline::RULE,
        super::posix_tools::bat_to_open::RULE,
        super::posix_tools::cat_to_open::RULE,
        super::posix_tools::date_to_date_now::RULE,
        super::posix_tools::df_to_sys_disks::RULE,
        super::posix_tools::redundant_echo::RULE,
        super::posix_tools::find_to_glob::RULE,
        super::posix_tools::free_to_sys_mem::RULE,
        super::posix_tools::grep_to_find_or_where::RULE,
        super::posix_tools::head_to_first::RULE,
        super::posix_tools::hostname_to_sys_host::RULE,
        super::posix_tools::external_cd_to_builtin::RULE,
        super::posix_tools::external_ls_to_builtin::RULE,
        super::posix_tools::pager_to_explore::RULE,
        super::posix_tools::read_to_input::RULE,
        super::posix_tools::sed_to_str_transform::RULE,
        super::posix_tools::external_sort_to_builtin::RULE,
        super::posix_tools::tac_to_reverse::RULE,
        super::posix_tools::tail_to_last::RULE,
        super::posix_tools::uname_to_sys_host::RULE,
        super::posix_tools::external_uniq_to_builtin::RULE,
        super::posix_tools::uptime_to_sys_host::RULE,
        super::posix_tools::users_to_sys_users::RULE,
        super::posix_tools::w_to_sys_users::RULE,
        super::posix_tools::wc_to_length::RULE,
        super::posix_tools::who_to_sys_users::RULE,
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
    description: "Replace common external CLI tools.",
    rules: &[
        super::external_tools::curl_to_http::RULE,
        super::external_tools::fd_to_glob::RULE,
        super::external_tools::jq_to_nu_pipeline::RULE,
        super::external_tools::wget_to_http_get::RULE,
        super::external_tools::external_which_to_builtin::RULE,
        super::structured_data_to_csv_tool::RULE,
        super::structured_data_to_json_tool::RULE,
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
    POSIX_TOOLS,
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
