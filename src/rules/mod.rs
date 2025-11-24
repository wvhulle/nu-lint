mod collapsible_if;
mod dangerous_file_operations;
mod descriptive_error_messages;
mod documentation;
mod error_make_metadata;
mod escape_string_interpolation_operators;
mod exit_only_in_main;
mod external_script_as_argument;
mod forbid_excessive_nesting;
mod inline_single_use_function;
mod redirection;
mod side_effects;
mod strong_typing;

mod max_function_body_length;
mod max_positional_params;
mod missing_stdin_in_shebang;
mod naming;

mod upstream;

mod bashisms;
mod prefer_compound_assignment;
mod prefer_direct_use;
mod prefer_error_make_for_stderr;
mod prefer_is_not_empty;
mod prefer_lines_over_split;
mod prefer_match_over_if_chain;
mod prefer_parse_command;
mod prefer_parse_over_each_split;
mod prefer_pipeline_input;
mod prefer_range_iteration;
mod prefer_try_for_error_handling;
mod prefer_where_over_each_if;
mod prefer_where_over_for_if;
mod print_exit_use_error_make;
mod remove_redundant_in;
mod row_condition_above_closure;
mod spacing;
mod systemd_journal_prefix;
mod unnecessary_mut;
mod unnecessary_variable_before_return;
mod unused_helper_functions;

use naming::{kebab_case_commands, screaming_snake_constants, snake_case_variables};
use spacing::{
    no_trailing_spaces, omit_list_commas, prefer_multiline_functions, prefer_multiline_lists,
    prefer_multiline_records,
};

use crate::rule::Rule;

pub const ALL_RULES: &[Rule] = &[
    bashisms::awk::rule(),
    bashisms::cat::rule(),
    bashisms::curl::rule(),
    bashisms::cut::rule(),
    bashisms::date::rule(),
    bashisms::echo::rule(),
    bashisms::exa::rule(),
    bashisms::eza::rule(),
    bashisms::fetch::rule(),
    bashisms::find::rule(),
    bashisms::grep::rule(),
    bashisms::head::rule(),
    bashisms::hostname::rule(),
    bashisms::jq::rule(),
    bashisms::ls::rule(),
    bashisms::man::rule(),
    bashisms::printenv::rule(),
    bashisms::read::rule(),
    bashisms::rg::rule(),
    bashisms::sed::rule(),
    bashisms::sort::rule(),
    bashisms::tail::rule(),
    bashisms::uniq::rule(),
    bashisms::wc::rule(),
    bashisms::wget::rule(),
    bashisms::which::rule(),
    collapsible_if::rule(),
    dangerous_file_operations::rule(),
    descriptive_error_messages::rule(),
    documentation::exported_function::rule(),
    documentation::main_named_args::rule(),
    documentation::main_positional_args::rule(),
    error_make_metadata::rule(),
    escape_string_interpolation_operators::rule(),
    exit_only_in_main::rule(),
    external_script_as_argument::rule(),
    forbid_excessive_nesting::rule(),
    inline_single_use_function::rule(),
    kebab_case_commands::rule(),
    max_function_body_length::rule(),
    max_positional_params::rule(),
    missing_stdin_in_shebang::rule(),
    no_trailing_spaces::rule(),
    omit_list_commas::rule(),
    prefer_compound_assignment::rule(),
    prefer_direct_use::rule(),
    prefer_error_make_for_stderr::rule(),
    prefer_is_not_empty::rule(),
    prefer_lines_over_split::rule(),
    prefer_match_over_if_chain::rule(),
    prefer_multiline_functions::rule(),
    prefer_multiline_lists::rule(),
    prefer_multiline_records::rule(),
    prefer_parse_command::rule(),
    prefer_parse_over_each_split::rule(),
    prefer_pipeline_input::rule(),
    prefer_range_iteration::rule(),
    prefer_try_for_error_handling::rule(),
    prefer_where_over_each_if::rule(),
    prefer_where_over_for_if::rule(),
    print_exit_use_error_make::rule(),
    redirection::check_complete_exit_code::rule(),
    redirection::prefer_complete_for_external_commands::rule(),
    redirection::prefer_complete_over_dev_null::rule(),
    redirection::redundant_ignore::rule(),
    remove_redundant_in::rule(),
    row_condition_above_closure::rule(),
    screaming_snake_constants::rule(),
    side_effects::mixed_io_types::rule(),
    side_effects::print_and_return_data::rule(),
    side_effects::pure_before_side_effects::rule(),
    snake_case_variables::rule(),
    spacing::brace_spacing::rule(),
    spacing::pipe_spacing::rule(),
    strong_typing::argument::rule(),
    strong_typing::paths::rule(),
    strong_typing::pipeline::rule(),
    systemd_journal_prefix::rule(),
    unnecessary_mut::rule(),
    unnecessary_variable_before_return::rule(),
    unused_helper_functions::rule(),
    upstream::nu_deprecated::rule(),
    upstream::nu_parse_error::rule(),
];
