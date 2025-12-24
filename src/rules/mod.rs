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

mod errors_to_stderr;
mod external_tools;
mod make_error_from_exit;
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
    avoid_nu_subprocess::RULE,
    avoid_self_import::RULE,
    check_complete_exit_code::RULE,
    collapsible_if::RULE,
    dangerous_file_operations::RULE,
    descriptive_error_messages::RULE,
    documentation::exported_function::RULE,
    documentation::main_named_args::RULE,
    documentation::main_positional_args::RULE,
    error_make_metadata::RULE,
    escape_string_interpolation_operators::RULE,
    exit_only_in_main::RULE,
    external_script_as_argument::RULE,
    external_tools::curl::RULE,
    external_tools::eza::RULE,
    external_tools::fd::RULE,
    external_tools::jq::RULE,
    external_tools::rg::RULE,
    external_tools::unnecessary_hat::RULE,
    external_tools::wget::RULE,
    external_tools::which::RULE,
    forbid_excessive_nesting::RULE,
    inline_single_use_function::RULE,
    max_function_body_length::RULE,
    max_positional_params::RULE,
    missing_stdin_in_shebang::RULE,
    naming::kebab_case_commands::RULE,
    naming::screaming_snake_constants::RULE,
    naming::snake_case_variables::RULE,
    posix_tools::awk::RULE,
    posix_tools::cat::RULE,
    posix_tools::cd::RULE,
    posix_tools::cut::RULE,
    posix_tools::date::RULE,
    posix_tools::echo::RULE,
    posix_tools::find::RULE,
    posix_tools::grep::RULE,
    posix_tools::head::RULE,
    posix_tools::ls::RULE,
    posix_tools::read::RULE,
    posix_tools::sed::RULE,
    posix_tools::sort::RULE,
    posix_tools::tail::RULE,
    posix_tools::uniq::RULE,
    posix_tools::wc::RULE,
    merge_get_cell_path::RULE,
    prefer_compound_assignment::RULE,
    prefer_direct_use::RULE,
    make_error_from_exit::RULE,
    prefer_is_not_empty::RULE,
    prefer_items_over_transpose::RULE,
    prefer_lines_over_split::RULE,
    replace_else_if_with_match::RULE,
    merge_multiline_print::RULE,
    prefer_parse_command::RULE,
    positional_to_pipeline::RULE,
    range_instead_of_for::RULE,
    prefer_subcommands_over_dispatch::RULE,
    try_instead_of_do::RULE,
    prefer_where_over_each_if::RULE,
    prefer_where_over_for_if::RULE,
    errors_to_stderr::RULE,
    non_final_failure_check::RULE,
    redirection::prefer_complete_over_dev_null::RULE,
    redirection::redundant_ignore::RULE,
    remove_redundant_in::RULE,
    row_condition_above_closure::RULE,
    side_effects::mixed_io_types::RULE,
    side_effects::print_and_return_data::RULE,
    spacing::brace_spacing::RULE,
    spacing::no_trailing_spaces::RULE,
    spacing::omit_list_commas::RULE,
    spacing::pipe_spacing::RULE,
    spacing::reflow_wide_pipelines::RULE,
    spacing::wrap_long_lists::RULE,
    spacing::wrap_records::RULE,
    strong_typing::argument::RULE,
    strong_typing::paths::RULE,
    strong_typing::pipeline::RULE,
    systemd::add_journal_prefix::RULE,
    systemd::mnemonic_log_level::RULE,
    unnecessary_mut::RULE,
    unnecessary_variable_before_return::RULE,
    unsafe_dynamic_record_access::RULE,
    unused_helper_functions::RULE,
    upstream::nu_deprecated::RULE,
    upstream::nu_parse_error::RULE,
];
