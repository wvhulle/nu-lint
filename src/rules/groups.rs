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
        super::avoid_last_exit_code::RULE,
        super::check_complete_exit_code::RULE,
        super::documentation::descriptive_error_messages::RULE,
        super::escape_string_interpolation_operators::RULE,
        super::exit_only_in_main::RULE,
        super::flag_compare_null::RULE,
        super::non_final_failure_check::RULE,
        super::error_make::non_fatal_catch::RULE,
        super::try_instead_of_do::RULE,
        super::unsafe_dynamic_record_access::RULE,
        super::missing_stdin_in_shebang::RULE,
        super::dynamic_script_import::RULE,
        super::catch_builtin_error_try::RULE,
        super::unchecked_cell_path_index::RULE,
        super::unchecked_get_index::RULE,
        super::unchecked_first_last::RULE,
        super::wrap_external_with_complete::RULE,
        super::use_over_source::RULE,
        super::spread_list_to_external::RULE,
    ],
};

const TYPE_SAFETY: Group = Group {
    name: "type-safety",
    description: "Annotate with type hints where possible.",
    rules: &[
        super::external_script_as_argument::RULE,
        super::nothing_outside_function_signature::RULE,
        super::typing::missing_argument_type::RULE,
        super::filesystem::string_as_path::RULE,
        super::typing::missing_output_type::RULE,
        super::typing::missing_in_type::RULE,
        super::avoid_nu_subprocess::RULE,
        super::dynamic_script_import::RULE,
    ],
};

const IDIOMATIC: Group = Group {
    name: "idioms",
    description: "Simplifications unique to the Nu language.",
    rules: &[
        super::builtin_not_empty::RULE,
        super::columns_in_to_has::RULE,
        super::columns_not_in_to_not_has::RULE,
        super::dispatch_with_subcommands::RULE,
        super::get_optional_to_has::RULE,
        super::get_optional_to_not_has::RULE,
        super::hardcoded_math_constants::RULE,
        super::items_instead_of_transpose_each::RULE,
        super::merge_get_cell_path::RULE,
        super::merge_multiline_print::RULE,
        super::positional_to_pipeline::RULE,
        super::use_over_source::RULE,
        super::shorten_with_compound_assignment::RULE,
        super::use_regex_operators::RULE,
        super::ansi_over_escape_codes::RULE,
        super::append_to_concat_assign::RULE,
        super::stdlib_log::RULE,
    ],
};

const PARSING: Group = Group {
    name: "parsing",
    description: "Better ways to parse and transform text data.",
    rules: &[
        super::parsing::lines_instead_of_split::RULE,
        super::never_space_split::RULE,
        super::parsing::lines_each_to_parse::RULE,
        super::parsing::simplify_regex::RULE,
        super::parsing::split_row_get_multistatement::RULE,
        super::parsing::split_row_first_last::RULE,
        super::parsing::split_row_get_inline::RULE,
    ],
};

const FILESYSTEM: Group = Group {
    name: "filesystem",
    description: "Simplify file and path operations.",
    rules: &[
        super::filesystem::from_after_parsed_open::RULE,
        super::filesystem::open_raw_from_to_open::RULE,
        super::filesystem::string_as_path::RULE,
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
        super::filtering::where_closure_to_it_condition::RULE,
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
        super::avoid_self_import::RULE,
        super::unnecessary_accumulate::RULE,
        super::unnecessary_variable_before_return::RULE,
        super::redundant_boolean_comparison::RULE,
        super::redundant_ignore::RULE,
        super::unnecessary_mut::RULE,
        super::unused_helper_functions::RULE,
        super::script_export_main::RULE,
        super::remove_string_quotes::RULE,
        super::inline_single_use_function::RULE,
        super::append_to_concat_assign::RULE,
    ],
};

const PERFORMANCE: Group = Group {
    name: "performance",
    description: "Rules with potential performance impact",
    rules: &[
        super::avoid_nu_subprocess::RULE,
        super::dispatch_with_subcommands::RULE,
        super::avoid_self_import::RULE,
        super::positional_to_pipeline::RULE,
        super::unnecessary_accumulate::RULE,
        super::merge_multiline_print::RULE,
        super::chained_str_replace::RULE,
        super::redundant_complete_streaming::RULE,
    ],
};

const POSIX_TOOLS: Group = Group {
    name: "posix",
    description: "Replace common bash/POSIX patterns.",
    rules: &[
        super::ignore_over_dev_null::RULE,
        super::posix_tools::awk::RULE,
        super::posix_tools::bat::RULE,
        super::posix_tools::cat::RULE,
        super::posix_tools::date::RULE,
        super::posix_tools::df::RULE,
        super::posix_tools::echo::RULE,
        super::posix_tools::find::RULE,
        super::posix_tools::free::RULE,
        super::posix_tools::grep::RULE,
        super::posix_tools::head::RULE,
        super::posix_tools::hostname::RULE,
        super::posix_tools::cd::RULE,
        super::posix_tools::ls::RULE,
        super::posix_tools::pagers::RULE,
        super::posix_tools::read::RULE,
        super::posix_tools::sed::RULE,
        super::posix_tools::sort::RULE,
        super::posix_tools::tac::RULE,
        super::posix_tools::tail::RULE,
        super::posix_tools::uname::RULE,
        super::posix_tools::uniq::RULE,
        super::posix_tools::uptime::RULE,
        super::posix_tools::users::RULE,
        super::posix_tools::w::RULE,
        super::posix_tools::wc::RULE,
        super::posix_tools::who::RULE,
    ],
};

const DOCUMENTATION: Group = Group {
    name: "documentation",
    description: "Improve actionability of user-facing messages.",
    rules: &[
        super::documentation::exported_function::RULE,
        super::documentation::descriptive_error_messages::RULE,
        super::error_make::add_label::RULE,
        super::error_make::add_help::RULE,
        super::error_make::add_span_to_label::RULE,
        super::error_make::add_url::RULE,
        super::documentation::main_positional_args::RULE,
        super::documentation::main_named_args::RULE,
        super::max_positional_params::RULE,
        super::prefer_long_flags::RULE,
    ],
};

const EXTERNAL_TOOLS: Group = Group {
    name: "external",
    description: "Replace common external CLI tools.",
    rules: &[
        super::external_tools::curl::RULE,
        super::external_tools::fd::RULE,
        super::external_tools::jq::RULE,
        super::external_tools::wget::RULE,
        super::external_tools::which::RULE,
    ],
};

const FORMATTING: Group = Group {
    name: "formatting",
    description: "Formatting according to Nushell guidelines.",
    rules: &[
        super::ansi_over_escape_codes::RULE,
        super::collapsible_if::RULE,
        super::forbid_excessive_nesting::RULE,
        super::max_function_body_length::RULE,
        super::replace_else_if_with_match::RULE,
        super::spacing::block_body_spacing::RULE,
        super::spacing::closure_param_spacing::RULE,
        super::spacing::no_trailing_spaces::RULE,
        super::spacing::omit_list_commas::RULE,
        super::spacing::pipe_spacing::RULE,
        super::spacing::record_brace_spacing::RULE,
        super::spacing::reflow_wide_pipelines::RULE,
        super::spacing::wrap_long_lists::RULE,
        super::spacing::wrap_records::RULE,
    ],
};

const NAMING: Group = Group {
    name: "naming",
    description: "Follow official naming conventions",
    rules: &[
        super::naming::kebab_case_commands::RULE,
        super::naming::screaming_snake_constants::RULE,
        super::naming::snake_case_variables::RULE,
        super::error_make::add_label::RULE,
    ],
};

const SIDE_EFFECTS: Group = Group {
    name: "effects",
    description: "Handle built-in and external commands with side-effects.",
    rules: &[
        super::dangerous_file_operations::RULE,
        super::errors_to_stderr::RULE,
        super::side_effects::mixed_io_types::RULE,
        super::side_effects::print_and_return_data::RULE,
        super::side_effects::silence_side_effect_only_each::RULE,
        super::side_effects::silence_stderr_data::RULE,
    ],
};

const UPSTREAM: Group = Group {
    name: "upstream",
    description: "Forward warnings and errors of the upstream Nushell parser.",
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

impl Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
