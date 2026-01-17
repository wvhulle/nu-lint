use crate::rule::Rule;

pub mod groups;

pub mod add_hat_external_commands;
pub mod ansi_over_escape_codes;
pub mod append_to_concat_assign;
pub mod assign_then_return;
pub mod catch_builtin_error_try;
pub mod chained_append;
pub mod chained_str_transform;
pub mod check_complete_exit_code;
pub mod check_typed_flag_before_use;
pub mod collapsible_if;
pub mod columns_in_to_has;
pub mod columns_not_in_to_not_has;
pub mod compound_assignment;
pub mod contains_to_regex_op;
pub mod custom_log_command;
pub mod dangerous_file_operations;
pub mod dispatch_with_subcommands;
pub mod do_not_compare_booleans;
pub mod documentation;
pub mod dynamic_script_import;
pub mod error_make;
pub mod errors_to_stderr;
pub mod exit_only_in_main;
pub mod explicit_long_flags;
pub mod external_script_as_argument;
pub mod external_tools;
pub mod filesystem;
pub mod filtering;
pub mod forbid_excessive_nesting;
pub mod fragile_last_exit_code;
pub mod get_optional_to_has;
pub mod get_optional_to_not_has;
pub mod glob_may_drop_quotes;
pub mod hardcoded_math_constants;
pub mod if_else_chain_to_match;
pub mod if_null_to_default;
pub mod ignore_over_dev_null;
pub mod list_param_to_variadic;
pub mod max_function_body_length;
pub mod max_positional_params;
pub mod merge_get_cell_path;
pub mod merge_multiline_print;
pub mod missing_stdin_in_shebang;
pub mod naming;
pub mod never_space_split;
pub mod non_final_failure_check;
pub mod not_is_empty_to_is_not_empty;
pub mod nothing_outside_signature;
pub mod parsing;
pub mod positional_to_pipeline;
pub mod posix_tools;
pub mod range_for_iteration;
pub mod redundant_ignore;
pub mod redundant_nu_subprocess;
pub mod remove_hat_not_builtin;
pub mod remove_redundant_in;
pub mod require_main_with_stdin;
pub mod script_export_main;
pub mod self_import;
pub mod side_effects;
pub mod single_call_command;
pub mod source_to_use;
pub mod spacing;
pub mod spread_list_to_external;
pub mod streaming_hidden_by_complete;
pub mod string_may_be_bare;
pub mod structured_data_to_csv_tool;
pub mod structured_data_to_json_tool;
pub mod transpose_items;
pub mod try_instead_of_do;
pub mod typing;
pub mod unchecked_cell_path_index;
pub mod unchecked_first_last;
pub mod unchecked_get_index;
pub mod unescaped_interpolation;
pub mod unnecessary_accumulate;
pub mod unnecessary_mut;
pub mod unsafe_dynamic_record_access;
pub mod unused_helper_functions;
pub mod upstream;
pub mod wrap_external_with_complete;

/// All rules that are used by default when linting.
pub const USED_RULES: &[&dyn Rule] = &[
    add_hat_external_commands::RULE,
    ansi_over_escape_codes::RULE,
    append_to_concat_assign::RULE,
    assign_then_return::RULE,
    catch_builtin_error_try::RULE,
    chained_append::RULE,
    chained_str_transform::RULE,
    check_complete_exit_code::RULE,
    check_typed_flag_before_use::RULE,
    collapsible_if::RULE,
    columns_in_to_has::RULE,
    columns_not_in_to_not_has::RULE,
    compound_assignment::RULE,
    contains_to_regex_op::RULE,
    custom_log_command::RULE,
    dangerous_file_operations::RULE,
    dispatch_with_subcommands::RULE,
    do_not_compare_booleans::RULE,
    documentation::add_doc_comment_exported_fn::RULE,
    documentation::descriptive_error_messages::RULE,
    documentation::main_named_args_docs::RULE,
    documentation::main_positional_args_docs::RULE,
    dynamic_script_import::RULE,
    error_make::add_help_to_error::RULE,
    error_make::add_label_to_error::RULE,
    error_make::add_span_to_label::RULE,
    error_make::add_url_to_error::RULE,
    error_make::error_make_for_non_fatal::RULE,
    errors_to_stderr::RULE,
    exit_only_in_main::RULE,
    explicit_long_flags::RULE,
    external_script_as_argument::RULE,
    external_tools::curl_to_http::RULE,
    external_tools::external_which_to_builtin::RULE,
    external_tools::fd_to_glob::RULE,
    external_tools::jq_to_nu_pipeline::RULE,
    external_tools::wget_to_http_get::RULE,
    filesystem::from_after_parsed_open::RULE,
    filesystem::open_raw_from_to_open::RULE,
    filesystem::string_param_as_path::RULE,
    filtering::each_if_to_where::RULE,
    filtering::for_filter_to_where::RULE,
    filtering::omit_it_in_row_condition::RULE,
    filtering::slice_to_drop::RULE,
    filtering::slice_to_last::RULE,
    filtering::slice_to_skip::RULE,
    filtering::slice_to_take::RULE,
    filtering::where_closure_drop_parameter::RULE,
    forbid_excessive_nesting::RULE,
    fragile_last_exit_code::RULE,
    get_optional_to_has::RULE,
    get_optional_to_not_has::RULE,
    glob_may_drop_quotes::RULE,
    hardcoded_math_constants::RULE,
    if_else_chain_to_match::RULE,
    if_null_to_default::RULE,
    ignore_over_dev_null::RULE,
    list_param_to_variadic::RULE,
    max_function_body_length::RULE,
    max_positional_params::RULE,
    merge_get_cell_path::RULE,
    merge_multiline_print::RULE,
    missing_stdin_in_shebang::RULE,
    naming::kebab_case_commands::RULE,
    naming::screaming_snake_constants::RULE,
    naming::snake_case_variables::RULE,
    never_space_split::RULE,
    non_final_failure_check::RULE,
    not_is_empty_to_is_not_empty::RULE,
    nothing_outside_signature::RULE,
    parsing::lines_each_to_parse::RULE,
    parsing::lines_instead_of_split::RULE,
    parsing::simplify_regex_parse::RULE,
    parsing::split_first_to_parse::RULE,
    parsing::split_row_get_inline::RULE,
    parsing::split_row_get_multistatement::RULE,
    parsing::split_row_space_to_split_words::RULE,
    positional_to_pipeline::RULE,
    posix_tools::awk_to_pipeline::RULE,
    posix_tools::bat_to_open::RULE,
    posix_tools::cat_to_open::RULE,
    posix_tools::date_to_date_now::RULE,
    posix_tools::df_to_sys_disks::RULE,
    posix_tools::external_cd_to_builtin::RULE,
    posix_tools::external_ls_to_builtin::RULE,
    posix_tools::external_sort_to_builtin::RULE,
    posix_tools::external_uniq_to_builtin::RULE,
    posix_tools::find_to_glob::RULE,
    posix_tools::free_to_sys_mem::RULE,
    posix_tools::grep_to_find_or_where::RULE,
    posix_tools::head_to_first::RULE,
    posix_tools::hostname_to_sys_host::RULE,
    posix_tools::pager_to_explore::RULE,
    posix_tools::read_to_input::RULE,
    posix_tools::redundant_echo::RULE,
    posix_tools::sed_to_str_transform::RULE,
    posix_tools::tac_to_reverse::RULE,
    posix_tools::tail_to_last::RULE,
    posix_tools::uname_to_sys_host::RULE,
    posix_tools::uptime_to_sys_host::RULE,
    posix_tools::users_to_sys_users::RULE,
    posix_tools::w_to_sys_users::RULE,
    posix_tools::wc_to_length::RULE,
    posix_tools::who_to_sys_users::RULE,
    range_for_iteration::loop_counter::RULE,
    range_for_iteration::while_counter::RULE,
    redundant_ignore::RULE,
    redundant_nu_subprocess::RULE,
    remove_hat_not_builtin::RULE,
    remove_redundant_in::RULE,
    require_main_with_stdin::RULE,
    script_export_main::RULE,
    self_import::RULE,
    side_effects::dont_mix_different_effects::RULE,
    side_effects::each_nothing_to_for_loop::RULE,
    side_effects::print_and_return_data::RULE,
    side_effects::silence_stderr_data::RULE,
    single_call_command::RULE,
    source_to_use::RULE,
    spacing::block_brace_spacing::RULE,
    spacing::closure_brace_pipe_spacing::RULE,
    spacing::closure_pipe_body_spacing::RULE,
    spacing::no_trailing_spaces::RULE,
    spacing::omit_list_commas::RULE,
    spacing::pipe_spacing::RULE,
    spacing::record_brace_spacing::RULE,
    spacing::reflow_wide_lists::RULE,
    spacing::reflow_wide_pipelines::RULE,
    spacing::wrap_wide_records::RULE,
    spread_list_to_external::RULE,
    streaming_hidden_by_complete::RULE,
    string_may_be_bare::RULE,
    structured_data_to_csv_tool::RULE,
    structured_data_to_json_tool::RULE,
    transpose_items::RULE,
    try_instead_of_do::RULE,
    typing::add_type_hints_arguments::RULE,
    typing::missing_in_type::RULE,
    typing::missing_output_type::RULE,
    unchecked_cell_path_index::RULE,
    unchecked_first_last::RULE,
    unchecked_get_index::RULE,
    unescaped_interpolation::RULE,
    unnecessary_accumulate::RULE,
    unnecessary_mut::RULE,
    unsafe_dynamic_record_access::RULE,
    unused_helper_functions::RULE,
    upstream::nu_deprecated::RULE,
    upstream::nu_parse_error::RULE,
    wrap_external_with_complete::RULE,
];
