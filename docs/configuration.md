# Configuration

Some rules are deactivated by default, usually because they are too noisy. You can activate them with a level override in the configuration file.

## Location

The configuration file is optional. By default nu-lint reads `nu-lint.toml` from your user configuration directory (`~/.config/nu-lint.toml` on Linux, `~/Library/Application Support/nu-lint.toml` on macOS, `%APPDATA%\nu-lint.toml` on Windows). Pass `--config <path>` to use a different file. The language server always reads the file in the user configuration directory and reloads it when it changes.

If the file cannot be parsed or names an unknown rule or group, the command line exits with an error, and the language server shows the error in your editor and uses the default levels until the file is fixed.

## Example

The following is the output of `nu-lint config example`. It lists every option with its default value, and every group and rule commented out (rules at their default level). Uncomment a line to change it. Write it to your configuration file as a starting point:

```bash
nu-lint config example | save ~/.config/nu-lint.toml
```

<!-- start-example-config -->
```toml
# Maximum length of a single-line pipeline before reflow_wide_pipelines splits it over multiple lines
max_pipeline_length = 80

# Where reflow_wide_pipelines puts the `|` when splitting a pipeline: "start" or "end" of a line
pipeline_placement = "start"

# Recommend `get --optional` instead of `$list.0?` for safe access
explicit_optional_access = false

# Maximum top-level statements in a function body before max_function_body_length fires
max_function_body_statements = 40

# Maximum pipe-separated steps a single-call function body may have before single_call_command stops suggesting it can be inlined
max_inlinable_pipeline_elements = 2

# Ignore parse errors in files that are imported but not linted themselves
skip_external_parse_errors = true

# Lint files one after another instead of in parallel
sequential = false

# Set the level of every rule in a group at once, for example to turn it off
[groups]
# Stop scripts that parse fine from failing or misbehaving at runtime.
# catch-runtime-failures = "off"
# Keep I/O, printing and dangerous file operations out of data-returning code.
# isolate-side-effects = "off"
# Add or tighten type annotations in signatures.
# annotate-types = "off"
# Replace verbose code with Nu operators, spread, assignment forms and the standard library.
# adopt-nu-syntax = "off"
# Turn loops, counters, slices and manual filters into pipeline commands.
# pipe-instead-of-loop = "off"
# Get structured data from text and files with `parse`, `lines` and `open`.
# parse-structured-data = "off"
# Swap external tools such as `grep`, `sed` and `jq` for equivalent built-ins.
# replace-external-tools = "off"
# Avoid subprocesses and buffering so data streams inside one Nu process.
# stream-in-process = "off"
# Delete unused or redundant code.
# remove-dead-code = "off"
# Flatten nesting, shorten bodies and simplify signatures and calls.
# reduce-complexity = "off"
# Fix spacing, line width and quoting according to the style guide.
# format-whitespace = "off"
# Apply the casing of the official style guide.
# follow-naming-conventions = "off"
# Add docs to commands and context to `error make`.
# document-interfaces = "off"
# Report errors and unhandled deprecations from nu-parser.
# forward-parser-diagnostics = "off"
# Rewrite syntax that nu-parser deprecated.
# migrate-deprecated-syntax = "off"

# Set the level of a single rule, overriding its group. Shown are the default levels.
[rules]

# catch-runtime-failures
# add_hat_external_commands = "off"
# fragile_last_exit_code = "warning"
# check_complete_exit_code = "warning"
# serialize_data_for_external = "warning"
# unescaped_interpolation = "error"
# exit_only_in_main = "warning"
# check_typed_flag_before_use = "warning"
# non_final_failure_check = "off"
# error_make_for_non_fatal = "hint"
# try_instead_of_do = "warning"
# unsafe_dynamic_record_access = "warning"
# missing_stdin_in_shebang = "error"
# dynamic_script_import = "hint"
# self_import = "error"
# catch_builtin_error_try = "warning"
# default_empty_string_masks_missing = "warning"
# unchecked_cell_path_index = "warning"
# unchecked_get_index = "warning"
# unhandled_external_error = "warning"
# spread_list_to_external = "warning"
# glob_may_drop_quotes = "warning"
# require_main_with_stdin = "error"

# isolate-side-effects
# dangerous_file_operations = "warning"
# errors_to_stderr = "warning"
# dont_mix_different_effects = "hint"
# print_and_return_data = "hint"
# each_nothing_to_for_loop = "hint"
# silence_stderr_data = "warning"

# annotate-types
# external_script_as_argument = "warning"
# nothing_outside_signature = "error"
# add_type_hints_arguments = "warning"
# string_param_as_path = "warning"
# missing_output_type = "warning"
# missing_in_type = "warning"

# adopt-nu-syntax
# not_is_empty_to_is_not_empty = "hint"
# columns_in_to_has = "warning"
# columns_not_in_to_not_has = "warning"
# dispatch_with_subcommands = "warning"
# get_optional_to_has = "warning"
# get_optional_to_not_has = "warning"
# hardcoded_math_constants = "hint"
# merge_get_cell_path = "hint"
# merge_multiline_print = "hint"
# source_to_use = "hint"
# compound_assignment = "hint"
# contains_to_regex_op = "hint"
# ansi_over_escape_codes = "hint"
# append_to_concat_assign = "warning"
# custom_log_command = "warning"
# chained_append = "hint"
# merge_with_record_spread = "hint"
# use_load_env = "hint"
# remove_hat_not_builtin = "hint"
# division_to_format_duration = "hint"
# ignore_over_dev_null = "warning"
# redundant_echo = "warning"
# if_null_to_default = "warning"
# list_param_to_variadic = "warning"

# pipe-instead-of-loop
# each_if_to_where = "warning"
# for_filter_to_where = "warning"
# omit_it_in_row_condition = "hint"
# slice_to_drop = "hint"
# slice_to_last = "hint"
# slice_to_skip = "hint"
# slice_to_take = "hint"
# uniq_to_set_operation = "hint"
# where_closure_drop_parameter = "hint"
# remove_redundant_in = "hint"
# loop_counter_to_range = "warning"
# while_counter_to_range = "warning"
# transpose_items = "hint"
# unnecessary_accumulate = "hint"

# parse-structured-data
# lines_instead_of_split = "warning"
# never_space_split = "hint"
# lines_each_to_parse = "hint"
# simplify_regex_parse = "hint"
# split_row_get_multistatement = "hint"
# split_first_to_parse = "hint"
# split_row_get_inline = "hint"
# split_row_space_to_split_words = "hint"
# from_after_parsed_open = "error"
# open_raw_from_to_open = "warning"

# replace-external-tools
# cat_to_open = "warning"
# curl_wget_to_http = "hint"
# cd_to_builtin = "warning"
# date_to_builtin = "warning"
# find_to_glob = "warning"
# grep_to_where = "warning"
# head_tail_to_first_last = "warning"
# jq_to_nu_pipeline = "warning"
# ls_to_builtin = "warning"
# read_to_input = "warning"
# sed_to_str_replace = "warning"
# sort_to_builtin = "warning"
# system_info_to_sys = "hint"
# tac_to_reverse = "warning"
# wc_to_length = "warning"

# stream-in-process
# redundant_nu_subprocess = "error"
# nu_script_to_run = "hint"
# positional_to_pipeline = "hint"
# chained_str_transform = "hint"
# streaming_hidden_by_complete = "off"

# remove-dead-code
# assign_then_return = "warning"
# do_not_compare_booleans = "warning"
# redundant_ignore = "hint"
# unnecessary_mut = "warning"
# unused_helper_functions = "warning"
# unused_parameter = "warning"
# unused_variable = "warning"
# script_export_main = "hint"

# reduce-complexity
# collapsible_if = "warning"
# forbid_excessive_nesting = "warning"
# max_function_body_length = "hint"
# if_else_chain_to_match = "warning"
# max_positional_params = "warning"
# explicit_long_flags = "hint"
# single_call_command = "hint"

# format-whitespace
# block_brace_spacing = "hint"
# closure_brace_pipe_spacing = "hint"
# closure_pipe_body_spacing = "hint"
# no_trailing_spaces = "off"
# omit_list_commas = "hint"
# pipe_spacing = "warning"
# record_brace_spacing = "hint"
# reflow_wide_pipelines = "hint"
# reflow_wide_lists = "hint"
# wrap_wide_records = "hint"
# string_may_be_bare = "hint"

# follow-naming-conventions
# kebab_case_commands = "hint"
# screaming_snake_constants = "hint"
# snake_case_variables = "warning"

# document-interfaces
# add_doc_comment_exported_fn = "hint"
# descriptive_error_messages = "hint"
# add_label_to_error = "hint"
# add_help_to_error = "off"
# add_span_to_label = "hint"
# add_url_to_error = "off"
# main_positional_args_docs = "hint"
# main_named_args_docs = "hint"

# forward-parser-diagnostics
# nu_deprecated = "warning"
# nu_parse_error = "error"

# migrate-deprecated-syntax
# filter_to_where = "warning"
# str_case_full_names = "warning"
# ignore_errors_to_optional = "warning"
# drop_get_sensitive = "warning"
# legacy_completer_inputs = "warning"
```
<!-- end-example-config -->

Options you leave out keep their default value. Levels set in `[rules]` take precedence over levels set in `[groups]`.

## Ignoring a single line

You can also turn off a rule violation on a particular line by appending a comment to the line:

```nu
$data | each {|row|
    $row.values.0 # nu-lint-ignore: unchecked_cell_path_index
}
```

There is a shortcut to do this by selecting the "ignore line violation" code action in the code action menu of your editor.
