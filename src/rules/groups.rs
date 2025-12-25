use core::fmt::{self, Display};

use crate::rule::Rule;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub name: &'static str,
    pub description: &'static str,
    pub rules: &'static [&'static dyn Rule],
}

const ERROR_HANDLING: Group = Group {
    name: "error-handling",
    description: "Error handling best practices",
    rules: &[
        super::check_complete_exit_code::RULE,
        super::documentation::descriptive_error_messages::RULE,
        super::escape_string_interpolation_operators::RULE,
        super::exit_only_in_main::RULE,
        super::missing_stdin_in_shebang::RULE,
        super::non_final_failure_check::RULE,
        super::error_make::use_error_make_for_catch::RULE,
        super::try_instead_of_do::RULE,
        super::errors_to_stderr::RULE,
        super::unsafe_dynamic_record_access::RULE,
    ],
};

const TYPE_SAFETY: Group = Group {
    name: "type-safety",
    description: "Encourage annotations with type hints.",
    rules: &[
        super::external_script_as_argument::RULE,
        super::strong_typing::argument::RULE,
        super::strong_typing::paths::RULE,
        super::strong_typing::pipeline::RULE,
        super::avoid_nu_subprocess::RULE,
    ],
};

const PERFORMANCE: Group = Group {
    name: "performance",
    description: "May yield performance improvements",
    rules: &[
        super::avoid_self_import::RULE,
        super::avoid_nu_subprocess::RULE,
        super::builtin_not_empty::RULE,
        super::dispatch_with_subcommands::RULE,
        super::shorten_with_compound_assignment::RULE,
        super::unnecessary_accumulate::RULE,
        super::lines_instead_of_split::RULE,
        super::parse_instead_of_split::RULE,
        super::positional_to_pipeline::RULE,
        super::range_for_iteration::while_counter::RULE,
        super::range_for_iteration::loop_counter::RULE,
        super::filter_with_where::over_each_if::RULE,
        super::filter_with_where::filter_collect::RULE,
        super::remove_redundant_in::RULE,
        super::row_condition_above_closure::RULE,
        super::unnecessary_variable_before_return::RULE,
        super::inline_single_use_function::RULE,
        super::items_instead_of_transpose_each::RULE,
        super::merge_get_cell_path::RULE,
        super::merge_multiline_print::RULE,
        super::redundant_ignore::RULE,
        super::unnecessary_mut::RULE,
        super::unused_helper_functions::RULE,
    ],
};

const SYSTEMD: Group = Group {
    name: "systemd",
    description: "When used in combination with systemd services",
    rules: &[
        super::systemd::add_journal_prefix::RULE,
        super::systemd::mnemonic_log_level::RULE,
    ],
};

const POSIX_TOOLS: Group = Group {
    name: "posix-tools",
    description: "Replace common bash/POSIX commands.",
    rules: &[
        super::ignore_over_dev_null::RULE,
        super::posix_tools::awk::RULE,
        super::posix_tools::cat::RULE,
        super::posix_tools::cut::RULE,
        super::posix_tools::date::RULE,
        super::posix_tools::echo::RULE,
        super::posix_tools::find::RULE,
        super::posix_tools::grep::RULE,
        super::posix_tools::head::RULE,
        super::posix_tools::cd::RULE,
        super::posix_tools::ls::RULE,
        super::posix_tools::read::RULE,
        super::posix_tools::sed::RULE,
        super::posix_tools::sort::RULE,
        super::posix_tools::tail::RULE,
        super::posix_tools::uniq::RULE,
        super::posix_tools::wc::RULE,
    ],
};

const DOCUMENTATION: Group = Group {
    name: "documentation",
    description: "Improve relevance of actionability of user-facing messages.",
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
    ],
};

const EXTERNAL_TOOLS: Group = Group {
    name: "external-tools",
    description: "Replace common external CLI tools.",
    rules: &[
        super::external_tools::curl::RULE,
        super::external_tools::eza::RULE,
        super::external_tools::fd::RULE,
        super::external_tools::jq::RULE,
        super::external_tools::rg::RULE,
        super::external_tools::unnecessary_hat::RULE,
        super::external_tools::wget::RULE,
        super::external_tools::which::RULE,
    ],
};

const FORMATTING: Group = Group {
    name: "formatting",
    description: "Formatting according to Nushell guidelines.",
    rules: &[
        super::collapsible_if::RULE,
        super::forbid_excessive_nesting::RULE,
        super::max_function_body_length::RULE,
        super::replace_else_if_with_match::RULE,
        super::spacing::brace_spacing::RULE,
        super::spacing::no_trailing_spaces::RULE,
        super::spacing::omit_list_commas::RULE,
        super::spacing::pipe_spacing::RULE,
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
    ],
};

const SIDE_EFFECTS: Group = Group {
    name: "side-effects",
    description: "Handle risky and unpredictable commands.",
    rules: &[
        super::dangerous_file_operations::RULE,
        super::side_effects::mixed_io_types::RULE,
        super::side_effects::print_and_return_data::RULE,
    ],
};

const UPSTREAM: Group = Group {
    name: "upstream",
    description: "Forward warnings and errors of the upstream Nushell parser.",
    rules: &[
        super::upstream::nu_deprecated::RULE,
        super::upstream::nu_parse_error::RULE,
    ],
};

pub const ALL_GROUPS: &[Group] = &[
    DOCUMENTATION,
    ERROR_HANDLING,
    EXTERNAL_TOOLS,
    FORMATTING,
    NAMING,
    PERFORMANCE,
    POSIX_TOOLS,
    SIDE_EFFECTS,
    SYSTEMD,
    TYPE_SAFETY,
    UPSTREAM,
];

impl Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
