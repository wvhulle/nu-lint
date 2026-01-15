use crate::rule::Rule;

pub mod groups;

pub mod always_annotate_ext_hat;
pub mod ansi_over_escape_codes;
pub mod append_to_concat_assign;
pub mod avoid_last_exit_code;
pub mod avoid_nu_subprocess;
pub mod avoid_self_import;
pub mod builtin_not_empty;
pub mod catch_builtin_error_try;
pub mod chained_append;
pub mod chained_str_replace;
pub mod check_complete_exit_code;
pub mod collapsible_if;
pub mod columns_in_to_has;
pub mod columns_not_in_to_not_has;
pub mod dangerous_file_operations;
pub mod dispatch_with_subcommands;
pub mod documentation;
pub mod dynamic_script_import;
pub mod error_make;
pub mod errors_to_stderr;
pub mod escape_string_interpolation_operators;
pub mod exit_only_in_main;
pub mod external_script_as_argument;
pub mod external_tools;
pub mod filesystem;
pub mod filtering;
pub mod flag_compare_null;
pub mod forbid_excessive_nesting;
pub mod get_optional_to_has;
pub mod get_optional_to_not_has;
pub mod glob_may_drop_quotes;
pub mod hardcoded_math_constants;
pub mod ignore_over_dev_null;
pub mod inline_single_use_function;
pub mod items_instead_of_transpose_each;
pub mod list_param_to_variadic;
pub mod max_function_body_length;
pub mod max_positional_params;
pub mod merge_get_cell_path;
pub mod merge_multiline_print;
pub mod missing_stdin_in_shebang;
pub mod naming;
pub mod never_space_split;
pub mod non_final_failure_check;
pub mod nothing_outside_function_signature;
pub mod parsing;
pub mod positional_to_pipeline;
pub mod posix_tools;
pub mod prefer_long_flags;
pub mod range_for_iteration;
pub mod redundant_complete_streaming;
pub mod redundant_ignore;
pub mod remove_hat_not_builtin;
pub mod remove_redundant_in;
pub mod remove_string_quotes;
pub mod replace_else_if_with_match;
pub mod require_main_with_stdin;
pub mod script_export_main;
pub mod shorten_with_compound_assignment;
pub mod side_effects;
pub mod spacing;
pub mod spread_list_to_external;
pub mod stdlib_log;
pub mod structured_data_to_csv_tool;
pub mod structured_data_to_json_tool;
pub mod try_instead_of_do;
pub mod typing;
pub mod unchecked_cell_path_index;
pub mod unchecked_first_last;
pub mod unchecked_get_index;
pub mod unnecessary_accumulate;
pub mod unnecessary_mut;
pub mod unnecessary_variable_before_return;
pub mod unsafe_dynamic_record_access;
pub mod unused_helper_functions;
pub mod upstream;
pub mod use_over_source;
pub mod use_regex_operators;
pub mod wrap_external_with_complete;

/// All rules that are used by default when linting.
pub const USED_RULES: &[&dyn Rule] = &[
    ansi_over_escape_codes::RULE,
    append_to_concat_assign::RULE,
    avoid_last_exit_code::RULE,
    avoid_nu_subprocess::RULE,
    avoid_self_import::RULE,
    remove_string_quotes::RULE,
    builtin_not_empty::RULE,
    chained_append::RULE,
    chained_str_replace::RULE,
    columns_in_to_has::RULE,
    columns_not_in_to_not_has::RULE,
    get_optional_to_has::RULE,
    get_optional_to_not_has::RULE,
    glob_may_drop_quotes::RULE,
    hardcoded_math_constants::RULE,
    check_complete_exit_code::RULE,
    collapsible_if::RULE,
    dangerous_file_operations::RULE,
    dispatch_with_subcommands::RULE,
    documentation::descriptive_error_messages::RULE,
    dynamic_script_import::RULE,
    documentation::exported_function::RULE,
    documentation::main_named_args::RULE,
    documentation::main_positional_args::RULE,
    error_make::add_help::RULE,
    error_make::add_label::RULE,
    error_make::add_span_to_label::RULE,
    error_make::add_url::RULE,
    error_make::non_fatal_catch::RULE,
    errors_to_stderr::RULE,
    escape_string_interpolation_operators::RULE,
    exit_only_in_main::RULE,
    external_script_as_argument::RULE,
    external_tools::curl::RULE,
    external_tools::fd::RULE,
    external_tools::jq::RULE,
    remove_hat_not_builtin::RULE,
    always_annotate_ext_hat::RULE,
    spread_list_to_external::RULE,
    external_tools::wget::RULE,
    external_tools::which::RULE,
    filtering::each_if_to_where::RULE,
    filtering::for_filter_to_where::RULE,
    filtering::omit_it_in_row_condition::RULE,
    filtering::slice_to_drop::RULE,
    filtering::slice_to_last::RULE,
    filtering::slice_to_skip::RULE,
    filtering::slice_to_take::RULE,
    filtering::where_closure_to_it_condition::RULE,
    flag_compare_null::RULE,
    forbid_excessive_nesting::RULE,
    ignore_over_dev_null::RULE,
    inline_single_use_function::RULE,
    items_instead_of_transpose_each::RULE,
    parsing::lines_instead_of_split::RULE,
    max_function_body_length::RULE,
    max_positional_params::RULE,
    merge_get_cell_path::RULE,
    merge_multiline_print::RULE,
    catch_builtin_error_try::RULE,
    missing_stdin_in_shebang::RULE,
    naming::kebab_case_commands::RULE,
    naming::screaming_snake_constants::RULE,
    naming::snake_case_variables::RULE,
    never_space_split::RULE,
    script_export_main::RULE,
    non_final_failure_check::RULE,
    nothing_outside_function_signature::RULE,
    filesystem::from_after_parsed_open::RULE,
    filesystem::open_raw_from_to_open::RULE,
    parsing::lines_each_to_parse::RULE,
    parsing::simplify_regex::RULE,
    parsing::split_row_get_multistatement::RULE,
    parsing::split_row_first_last::RULE,
    parsing::split_row_get_inline::RULE,
    positional_to_pipeline::RULE,
    list_param_to_variadic::RULE,
    prefer_long_flags::RULE,
    use_over_source::RULE,
    posix_tools::awk::RULE,
    posix_tools::bat::RULE,
    posix_tools::cat::RULE,
    posix_tools::cd::RULE,
    posix_tools::date::RULE,
    posix_tools::df::RULE,
    posix_tools::echo::RULE,
    posix_tools::find::RULE,
    posix_tools::free::RULE,
    posix_tools::grep::RULE,
    posix_tools::head::RULE,
    posix_tools::hostname::RULE,
    posix_tools::ls::RULE,
    posix_tools::pagers::RULE,
    posix_tools::read::RULE,
    posix_tools::sed::RULE,
    posix_tools::sort::RULE,
    posix_tools::tac::RULE,
    posix_tools::tail::RULE,
    posix_tools::uname::RULE,
    posix_tools::uniq::RULE,
    posix_tools::uptime::RULE,
    posix_tools::users::RULE,
    posix_tools::w::RULE,
    posix_tools::wc::RULE,
    posix_tools::who::RULE,
    range_for_iteration::loop_counter::RULE,
    range_for_iteration::while_counter::RULE,
    redundant_ignore::RULE,
    remove_redundant_in::RULE,
    replace_else_if_with_match::RULE,
    require_main_with_stdin::RULE,
    shorten_with_compound_assignment::RULE,
    side_effects::mixed_io_types::RULE,
    side_effects::print_and_return_data::RULE,
    side_effects::silence_side_effect_only_each::RULE,
    side_effects::silence_stderr_data::RULE,
    spacing::block_body_spacing::RULE,
    spacing::closure_body_spacing::RULE,
    spacing::closure_param_spacing::RULE,
    spacing::no_trailing_spaces::RULE,
    spacing::omit_list_commas::RULE,
    spacing::pipe_spacing::RULE,
    spacing::record_brace_spacing::RULE,
    spacing::reflow_wide_pipelines::RULE,
    spacing::wrap_long_lists::RULE,
    spacing::wrap_records::RULE,
    structured_data_to_csv_tool::RULE,
    structured_data_to_json_tool::RULE,
    typing::missing_argument_type::RULE,
    filesystem::string_as_path::RULE,
    typing::missing_output_type::RULE,
    typing::missing_in_type::RULE,
    try_instead_of_do::RULE,
    unnecessary_accumulate::RULE,
    unnecessary_mut::RULE,
    unnecessary_variable_before_return::RULE,
    unsafe_dynamic_record_access::RULE,
    unchecked_cell_path_index::RULE,
    unchecked_first_last::RULE,
    unchecked_get_index::RULE,
    unused_helper_functions::RULE,
    upstream::nu_deprecated::RULE,
    upstream::nu_parse_error::RULE,
    use_regex_operators::RULE,
    // redundant_complete_streaming::RULE, // TODO: test this new rule
    wrap_external_with_complete::RULE,
    stdlib_log::RULE,
];
