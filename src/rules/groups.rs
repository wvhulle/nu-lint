use core::fmt::{self, Display};

use crate::rule::Rule;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub name: &'static str,
    pub description: &'static str,
    pub rules: &'static [Rule],
}

const ERROR_HANDLING: Group = Group {
    name: "error-handling",
    description: "Error handling best practices",
    rules: &[
        super::error_make::add_label::RULE,
        super::error_make::add_help::RULE,
        super::error_make::add_span_to_label::RULE,
        super::error_make::add_url::RULE,
        super::check_complete_exit_code::RULE,
        super::documentation::descriptive_error_messages::RULE,
        super::escape_string_interpolation_operators::RULE,
        super::non_final_failure_check::RULE,
        super::make_error_from_exit::RULE,
        super::try_instead_of_do::RULE,
        super::errors_to_stderr::RULE,
    ],
};

const TYPE_SAFETY: Group = Group {
    name: "type-safety",
    description: "Enforce explicit typing of variables and pipelines.",
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
    description: "Performance optimization hints",
    rules: &[
        super::avoid_self_import::RULE,
        super::avoid_nu_subprocess::RULE,
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
        super::unnecessary_variable_before_return::RULE,
    ],
};

const SYSTEMD: Group = Group {
    name: "systemd",
    description: "Rules for systemd service scripts",
    rules: &[
        super::systemd::add_journal_prefix::RULE,
        super::systemd::mnemonic_log_level::RULE,
    ],
};

const POSIX_TOOLS: Group = Group {
    name: "posix-tools",
    description: "Replace common bash/POSIX commands with native Nushell equivalents",
    rules: &[
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
    description: "Documentation quality rules",
    rules: &[
        super::documentation::exported_function::RULE,
        super::documentation::descriptive_error_messages::RULE,
        super::error_make::add_label::RULE,
        super::error_make::add_help::RULE,
        super::error_make::add_span_to_label::RULE,
        super::error_make::add_url::RULE,
        super::documentation::main_positional_args::RULE,
        super::documentation::main_named_args::RULE,
    ],
};

const EXTERNAL_TOOLS: Group = Group {
    name: "external-tools",
    description: "Replace modern CLI tools with native Nushell equivalents",
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
    description: "Check that code is formatted according to the official Nushell guidelines.",
    rules: &[
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
    description: "Linting rules for naming conventions",
    rules: &[
        super::naming::kebab_case_commands::RULE,
        super::naming::screaming_snake_constants::RULE,
        super::naming::snake_case_variables::RULE,
    ],
};

const SIDE_EFFECTS: Group = Group {
    name: "side-effects",
    description: "Side effects (or effects) are things commands do that escape the type system, \
                  but happen often and may cause unexpected behavior.",
    rules: &[
        super::side_effects::mixed_io_types::RULE,
        super::side_effects::print_and_return_data::RULE,
    ],
};

pub const ALL_GROUPS: &[Group] = &[
    POSIX_TOOLS,
    DOCUMENTATION,
    ERROR_HANDLING,
    EXTERNAL_TOOLS,
    FORMATTING,
    NAMING,
    PERFORMANCE,
    SIDE_EFFECTS,
    SYSTEMD,
    TYPE_SAFETY,
];

impl Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
