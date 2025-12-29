use crate::rule::Rule;

pub mod groups;

mod avoid_nu_subprocess;
mod avoid_self_import;
mod bare_string_okay;
mod builtin_not_empty;
pub mod check_complete_exit_code;
mod collapsible_if;
mod dangerous_file_operations;
mod dispatch_with_subcommands;
mod documentation;
mod error_make;
mod errors_to_stderr;
mod escape_string_interpolation_operators;
mod exit_only_in_main;
mod external_script_as_argument;
mod external_tools;
mod filter_with_where;
mod forbid_excessive_nesting;
mod ignore_over_dev_null;
mod inline_single_use_function;
mod items_instead_of_transpose_each;
mod lines_instead_of_split;
mod max_function_body_length;
mod max_positional_params;
mod merge_get_cell_path;
mod merge_multiline_print;
mod missing_stdin_in_shebang;
mod naming;
mod never_space_split;
mod no_export_main;
mod non_final_failure_check;
mod parse_instead_of_split;
mod positional_to_pipeline;
mod posix_tools;
mod range_for_iteration;

pub mod redundant_ignore;
mod remove_redundant_in;
mod replace_else_if_with_match;
mod row_condition_above_closure;
mod shorten_with_compound_assignment;
mod side_effects;
mod spacing;
mod strong_typing;
mod systemd;

mod try_instead_of_do;
mod unnecessary_accumulate;
mod unnecessary_mut;
mod unnecessary_variable_before_return;
mod unsafe_dynamic_record_access;
mod unused_helper_functions;
mod upstream;

pub const ALL_RULES: &[&dyn Rule] = &[
    avoid_nu_subprocess::RULE,
    avoid_self_import::RULE,
    bare_string_okay::RULE,
    check_complete_exit_code::RULE,
    collapsible_if::RULE,
    dangerous_file_operations::RULE,
    documentation::descriptive_error_messages::RULE,
    documentation::exported_function::RULE,
    documentation::main_named_args::RULE,
    documentation::main_positional_args::RULE,
    error_make::add_help::RULE,
    error_make::add_label::RULE,
    error_make::add_span_to_label::RULE,
    error_make::add_url::RULE,
    errors_to_stderr::RULE,
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
    side_effects::silence_side_effect_only_each::RULE,
    inline_single_use_function::RULE,
    items_instead_of_transpose_each::RULE,
    lines_instead_of_split::RULE,
    error_make::use_error_make_for_catch::RULE,
    max_function_body_length::RULE,
    max_positional_params::RULE,
    merge_get_cell_path::RULE,
    merge_multiline_print::RULE,
    missing_stdin_in_shebang::RULE,
    naming::kebab_case_commands::RULE,
    naming::screaming_snake_constants::RULE,
    naming::snake_case_variables::RULE,
    never_space_split::RULE,
    no_export_main::RULE,
    non_final_failure_check::RULE,
    parse_instead_of_split::RULE,
    positional_to_pipeline::RULE,
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
    shorten_with_compound_assignment::RULE,
    unnecessary_accumulate::RULE,
    builtin_not_empty::RULE,
    dispatch_with_subcommands::RULE,
    range_for_iteration::loop_counter::RULE,
    range_for_iteration::while_counter::RULE,
    filter_with_where::over_each_if::RULE,
    filter_with_where::filter_collect::RULE,
    ignore_over_dev_null::RULE,
    redundant_ignore::RULE,
    remove_redundant_in::RULE,
    replace_else_if_with_match::RULE,
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
    try_instead_of_do::RULE,
    unnecessary_mut::RULE,
    unnecessary_variable_before_return::RULE,
    unsafe_dynamic_record_access::RULE,
    unused_helper_functions::RULE,
    upstream::nu_deprecated::RULE,
    upstream::nu_parse_error::RULE,
];
