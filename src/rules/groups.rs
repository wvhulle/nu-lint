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
        super::error_make_metadata::rule(),
        super::check_complete_exit_code::rule(),
        super::descriptive_error_messages::rule(),
        super::escape_string_interpolation_operators::rule(),
        super::non_final_failure_check::rule(),
        super::combine_print_stderr_exit::rule(),
        super::try_instead_of_do::rule(),
        super::print_exit_use_error_make::rule(),
    ],
};

const TYPE_SAFETY: Group = Group {
    name: "type-safety",
    description: "Enforce explicit typing of variables and pipelines.",
    rules: &[
        super::external_script_as_argument::rule(),
        super::strong_typing::argument::rule(),
        super::strong_typing::paths::rule(),
        super::strong_typing::pipeline::rule(),
        super::avoid_nu_subprocess::rule(),
    ],
};

const PERFORMANCE: Group = Group {
    name: "performance",
    description: "Performance optimization hints",
    rules: &[
        super::avoid_self_import::rule(),
        super::avoid_nu_subprocess::rule(),
        super::prefer_compound_assignment::rule(),
        super::prefer_direct_use::rule(),
        super::prefer_lines_over_split::rule(),
        super::prefer_parse_command::rule(),
        super::positional_to_pipeline::rule(),
        super::range_instead_of_for::rule(),
        super::prefer_where_over_each_if::rule(),
        super::prefer_where_over_for_if::rule(),
        super::remove_redundant_in::rule(),
        super::unnecessary_variable_before_return::rule(),
    ],
};

const SYSTEMD: Group = Group {
    name: "systemd",
    description: "Rules for systemd service scripts",
    rules: &[
        super::systemd::add_journal_prefix::rule(),
        super::systemd::mnemonic_log_level::rule(),
    ],
};

const POSIX_TOOLS: Group = Group {
    name: "posix-tools",
    description: "Replace common bash/POSIX commands with native Nushell equivalents",
    rules: &[
        super::posix_tools::awk::rule(),
        super::posix_tools::cat::rule(),
        super::posix_tools::cut::rule(),
        super::posix_tools::date::rule(),
        super::posix_tools::echo::rule(),
        super::posix_tools::find::rule(),
        super::posix_tools::grep::rule(),
        super::posix_tools::head::rule(),
        super::posix_tools::cd::rule(),
        super::posix_tools::ls::rule(),
        super::posix_tools::read::rule(),
        super::posix_tools::sed::rule(),
        super::posix_tools::sort::rule(),
        super::posix_tools::tail::rule(),
        super::posix_tools::uniq::rule(),
        super::posix_tools::wc::rule(),
    ],
};

const DOCUMENTATION: Group = Group {
    name: "documentation",
    description: "Documentation quality rules",
    rules: &[
        super::documentation::exported_function::rule(),
        super::descriptive_error_messages::rule(),
        super::documentation::main_positional_args::rule(),
        super::documentation::main_named_args::rule(),
    ],
};

const EXTERNAL_TOOLS: Group = Group {
    name: "external-tools",
    description: "Replace modern CLI tools with native Nushell equivalents",
    rules: &[
        super::external_tools::curl::rule(),
        super::external_tools::eza::rule(),
        super::external_tools::fd::rule(),
        super::external_tools::jq::rule(),
        super::external_tools::rg::rule(),
        super::external_tools::unnecessary_hat::rule(),
        super::external_tools::wget::rule(),
        super::external_tools::which::rule(),
    ],
};

const FORMATTING: Group = Group {
    name: "formatting",
    description: "Check that code is formatted according to the official Nushell guidelines.",
    rules: &[
        super::spacing::brace_spacing::rule(),
        super::spacing::no_trailing_spaces::rule(),
        super::spacing::omit_list_commas::rule(),
        super::spacing::pipe_spacing::rule(),
        super::spacing::reflow_wide_pipelines::rule(),
        super::spacing::wrap_long_lists::rule(),
        super::spacing::prefer_multiline_records::rule(),
    ],
};

const NAMING: Group = Group {
    name: "naming",
    description: "Linting rules for naming conventions",
    rules: &[
        super::naming::kebab_case_commands::rule(),
        super::naming::screaming_snake_constants::rule(),
        super::naming::snake_case_variables::rule(),
    ],
};

const SIDE_EFFECTS: Group = Group {
    name: "side-effects",
    description: "Side effects (or effects) are things commands do that escape the type system, \
                  but happen often and may cause unexpected behavior.",
    rules: &[
        super::side_effects::mixed_io_types::rule(),
        super::side_effects::print_and_return_data::rule(),
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
