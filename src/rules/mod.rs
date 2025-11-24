mod check_complete_exit_code;
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
mod side_effects;
mod strong_typed_paths;
mod unnecessary_ignore;

mod max_function_body_length;
mod max_positional_params;
mod missing_stdin_in_shebang;
mod missing_type_annotations;
mod naming;

mod nu_deprecated;
mod nu_parse_error;

mod prefer_complete_for_external_commands;

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
mod replace;
mod row_condition_above_closure;
mod spacing;
mod systemd_journal_prefix;
mod unnecessary_mut;
mod unnecessary_variable_before_return;
mod unused_helper_functions;
mod unused_output;

use naming::{kebab_case_commands, screaming_snake_constants, snake_case_variables};
use spacing::{
    no_trailing_spaces, omit_list_commas, prefer_multiline_functions, prefer_multiline_lists,
    prefer_multiline_records,
};

use crate::rule::Rule;

pub const ALL_RULES: &[Rule] = &[
    // echo rule already included above; remove duplicate
    check_complete_exit_code::rule(),
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
    missing_type_annotations::argument::rule(),
    missing_type_annotations::pipeline::rule(),
    no_trailing_spaces::rule(),
    nu_deprecated::rule(),
    nu_parse_error::rule(),
    omit_list_commas::rule(),
    prefer_complete_for_external_commands::rule(),
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
    remove_redundant_in::rule(),
    replace::awk::rule(),
    replace::cat::rule(),
    replace::curl::rule(),
    replace::cut::rule(),
    replace::date::rule(),
    replace::echo::rule(),
    replace::exa::rule(),
    replace::eza::rule(),
    replace::fetch::rule(),
    replace::find::rule(),
    replace::grep::rule(),
    replace::head::rule(),
    replace::hostname::rule(),
    replace::jq::rule(),
    replace::ls::rule(),
    replace::man::rule(),
    replace::printenv::rule(),
    replace::read::rule(),
    replace::rg::rule(),
    replace::sed::rule(),
    replace::sort::rule(),
    replace::tail::rule(),
    replace::uniq::rule(),
    replace::wc::rule(),
    replace::wget::rule(),
    replace::which::rule(),
    row_condition_above_closure::rule(),
    screaming_snake_constants::rule(),
    side_effects::mixed_io_types::rule(),
    side_effects::print_and_return_data::rule(),
    side_effects::pure_before_side_effects::rule(),
    snake_case_variables::rule(),
    spacing::brace_spacing::rule(),
    spacing::pipe_spacing::rule(),
    strong_typed_paths::rule(),
    systemd_journal_prefix::rule(),
    unnecessary_ignore::rule(),
    unnecessary_mut::rule(),
    unnecessary_variable_before_return::rule(),
    unused_helper_functions::rule(),
    unused_output::rule(),
];
