use crate::rule::Rule;

pub mod groups;

mod avoid_nu_subprocess;
mod avoid_self_import;
pub mod check_complete_exit_code;
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
mod non_final_failure_check;
mod redirection;
mod side_effects;
mod strong_typing;

mod max_function_body_length;
mod max_positional_params;
mod missing_stdin_in_shebang;
mod naming;

mod combine_print_stderr_exit;
mod external_tools;
mod merge_get_cell_path;
mod merge_multiline_print;
mod positional_to_pipeline;
mod posix_tools;
mod prefer_compound_assignment;
mod prefer_direct_use;
mod prefer_is_not_empty;
mod prefer_items_over_transpose;
mod prefer_lines_over_split;
mod prefer_parse_command;
mod prefer_subcommands_over_dispatch;
mod prefer_where_over_each_if;
mod prefer_where_over_for_if;
mod print_exit_use_error_make;
mod range_instead_of_for;
mod remove_redundant_in;
mod replace_else_if_with_match;
mod row_condition_above_closure;
mod spacing;
mod systemd;
mod try_instead_of_do;
mod unnecessary_mut;
mod unnecessary_variable_before_return;
mod unsafe_dynamic_record_access;
mod unused_helper_functions;
mod upstream;

pub const ALL_RULES: &[Rule] = &[
    avoid_nu_subprocess::rule(),
    avoid_self_import::rule(),
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
    external_tools::curl::rule(),
    external_tools::eza::rule(),
    external_tools::fd::rule(),
    external_tools::jq::rule(),
    external_tools::rg::rule(),
    external_tools::unnecessary_hat::rule(),
    external_tools::wget::rule(),
    external_tools::which::rule(),
    forbid_excessive_nesting::rule(),
    inline_single_use_function::rule(),
    max_function_body_length::rule(),
    max_positional_params::rule(),
    missing_stdin_in_shebang::rule(),
    naming::kebab_case_commands::rule(),
    naming::screaming_snake_constants::rule(),
    naming::snake_case_variables::rule(),
    posix_tools::awk::rule(),
    posix_tools::cat::rule(),
    posix_tools::cd::rule(),
    posix_tools::cut::rule(),
    posix_tools::date::rule(),
    posix_tools::echo::rule(),
    posix_tools::find::rule(),
    posix_tools::grep::rule(),
    posix_tools::head::rule(),
    posix_tools::ls::rule(),
    posix_tools::read::rule(),
    posix_tools::sed::rule(),
    posix_tools::sort::rule(),
    posix_tools::tail::rule(),
    posix_tools::uniq::rule(),
    posix_tools::wc::rule(),
    merge_get_cell_path::rule(),
    prefer_compound_assignment::rule(),
    prefer_direct_use::rule(),
    combine_print_stderr_exit::rule(),
    prefer_is_not_empty::rule(),
    prefer_items_over_transpose::rule(),
    prefer_lines_over_split::rule(),
    replace_else_if_with_match::rule(),
    merge_multiline_print::rule(),
    prefer_parse_command::rule(),
    positional_to_pipeline::rule(),
    range_instead_of_for::rule(),
    prefer_subcommands_over_dispatch::rule(),
    try_instead_of_do::rule(),
    prefer_where_over_each_if::rule(),
    prefer_where_over_for_if::rule(),
    print_exit_use_error_make::rule(),
    non_final_failure_check::rule(),
    redirection::prefer_complete_over_dev_null::rule(),
    redirection::redundant_ignore::rule(),
    remove_redundant_in::rule(),
    row_condition_above_closure::rule(),
    side_effects::mixed_io_types::rule(),
    side_effects::print_and_return_data::rule(),
    spacing::brace_spacing::rule(),
    spacing::no_trailing_spaces::rule(),
    spacing::omit_list_commas::rule(),
    spacing::pipe_spacing::rule(),
    spacing::reflow_wide_pipelines::rule(),
    spacing::wrap_long_lists::rule(),
    spacing::prefer_multiline_records::rule(),
    strong_typing::argument::rule(),
    strong_typing::paths::rule(),
    strong_typing::pipeline::rule(),
    systemd::add_journal_prefix::rule(),
    systemd::mnemonic_log_level::rule(),
    unnecessary_mut::rule(),
    unnecessary_variable_before_return::rule(),
    unsafe_dynamic_record_access::rule(),
    unused_helper_functions::rule(),
    upstream::nu_deprecated::rule(),
    upstream::nu_parse_error::rule(),
];
