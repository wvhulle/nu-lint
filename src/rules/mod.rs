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
mod unnecessary_ignore;

mod max_function_body_length;
mod max_positional_params;
mod missing_type_annotations;
mod naming;

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
mod prefer_where_over_each_if;
mod prefer_where_over_for_if;
mod print_exit_use_error_make;
mod remove_redundant_in;
mod replace_by_builtin;
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

pub const ALL_RULES: [Rule; 66] = [
    check_complete_exit_code::rule(),
    collapsible_if::rule(),
    dangerous_file_operations::rule(),
    descriptive_error_messages::rule(),
    error_make_metadata::rule(),
    escape_string_interpolation_operators::rule(),
    exit_only_in_main::rule(),
    unnecessary_ignore::rule(),
    documentation::exported_function::rule(),
    documentation::main_positional_args::rule(),
    documentation::main_named_args::rule(),
    forbid_excessive_nesting::rule(),
    external_script_as_argument::rule(),
    inline_single_use_function::rule(),
    side_effects::mixed_io_types::rule(),
    side_effects::print_and_return_data::rule(),
    side_effects::pure_before_side_effects::rule(),
    kebab_case_commands::rule(),
    max_function_body_length::rule(),
    max_positional_params::rule(),
    missing_type_annotations::argument::rule(),
    missing_type_annotations::pipeline::rule(),
    prefer_multiline_functions::rule(),
    prefer_multiline_lists::rule(),
    prefer_multiline_records::rule(),
    replace_by_builtin::echo::rule(),
    no_trailing_spaces::rule(),
    nu_parse_error::rule(),
    omit_list_commas::rule(),
    prefer_complete_for_external_commands::rule(),
    replace_by_builtin::cat::rule(),
    replace_by_builtin::find::rule(),
    replace_by_builtin::grep::rule(),
    replace_by_builtin::head::rule(),
    replace_by_builtin::http::rule(),
    replace_by_builtin::jq::rule(),
    replace_by_builtin::ls::rule(),
    replace_by_builtin::other::rule(),
    replace_by_builtin::sed::rule(),
    replace_by_builtin::sort::rule(),
    replace_by_builtin::tail::rule(),
    replace_by_builtin::uniq::rule(),
    prefer_compound_assignment::rule(),
    prefer_direct_use::rule(),
    prefer_error_make_for_stderr::rule(),
    print_exit_use_error_make::rule(),
    prefer_is_not_empty::rule(),
    prefer_lines_over_split::rule(),
    prefer_match_over_if_chain::rule(),
    prefer_parse_command::rule(),
    prefer_parse_over_each_split::rule(),
    replace_by_builtin::path::rule(),
    prefer_pipeline_input::rule(),
    prefer_range_iteration::rule(),
    prefer_where_over_each_if::rule(),
    prefer_where_over_for_if::rule(),
    remove_redundant_in::rule(),
    screaming_snake_constants::rule(),
    snake_case_variables::rule(),
    spacing::brace_spacing::rule(),
    spacing::pipe_spacing::rule(),
    systemd_journal_prefix::rule(),
    unnecessary_mut::rule(),
    unnecessary_variable_before_return::rule(),
    unused_helper_functions::rule(),
    unused_output::rule(),
];
