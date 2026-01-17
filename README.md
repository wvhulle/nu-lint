# Nu-Lint

Linter for the innovative [Nu](https://www.nushell.sh/) shell.

Learning to use a new shell is a radical change that can use some assistance. This project is aimed at helping new and intermediate users of the [Nu](https://www.nushell.sh/) shell. Nu shell has a lot of useful features not found in other scripting languages. This linter will give you hints to use all of them and even offer automatic fixes.

## Example

The rule `turn_positional_into_stream_input` recommends to use pipelines instead of positional arguments:

```nu
def filter-positive [numbers] {
    $numbers | where $it > 0
}
```

```nu
def filter-positive [] {
    where $it > 0
}
```

This encourages lazy pipeline input: a positional list argument loads all data into memory at once, while implicit pipeline input processes elements one at a time.

## Usage

Lint all Nu files in working directory with:

```bash
nu-lint
```

To see all options and get help:

```bash
nu-lint --help
```

## Rules

All rules are optional and can be disabled with a configuration file. The rule definitions are compatible with:

- The official Nu parser [nu-parser](https://crates.io/crates/nu-parser).
- The Tree-sitter-based Nu formatter [topiary-nushell](https://github.com/blindFS/topiary-nushell).
- The official Nu [style guide](https://www.nushell.sh/book/style_guide.html)

Some of the rules need further testing and improvement. Please make an issue on the issue tracker to report any bugs. In early stages of development some rules may be replaced or renamed.

+140 rules are defined and most have automatic fixes available (list may be out-of-date):

<!-- start-rule-groups -->
`idioms` - Simplifications unique to the Nu language.

- `use_is_not_empty` (auto-fix)
- `columns_in_to_has` (auto-fix)
- `columns_not_in_to_not_has` (auto-fix)
- `dispatch_with_subcommands`
- `get_optional_to_has` (auto-fix)
- `get_optional_to_not_has` (auto-fix)
- `hardcoded_math_constants` (auto-fix)
- `transpose_items` (auto-fix)
- `merge_get_cell_path` (auto-fix)
- `merge_multiline_print` (auto-fix)
- `turn_positional_into_stream_input` (auto-fix)
- `use_over_source`
- `shorten_with_compound_assignment` (auto-fix)
- `contains_to_like_regex_operators` (auto-fix)
- `ansi_over_escape_codes` (auto-fix)
- `append_to_concat_assign` (auto-fix)
- `custom_log_command` (auto-fix)

`parsing` - Better ways to parse and transform text data.

- `lines_instead_of_split` (auto-fix)
- `never_space_split` (auto-fix)
- `lines_each_to_parse` (auto-fix)
- `simplify_regex_parse` (auto-fix)
- `split_row_get_multistatement` (auto-fix)
- `split_first_to_parse` (auto-fix)
- `split_row_get_inline` (auto-fix)
- `split_row_space_to_split_words` (auto-fix)

`filesystem` - Simplify file and path operations.

- `from_after_parsed_open` (auto-fix)
- `open_raw_from_to_open` (auto-fix)
- `string_param_as_path` (auto-fix)

`dead-code` - Remove unused or redundant code

- `avoid_self_import`
- `unnecessary_accumulate`
- `unnecessary_variable_before_return` (auto-fix)
- `do_not_compare_booleans` (auto-fix)
- `if_null_to_default` (auto-fix)
- `redundant_ignore` (auto-fix)
- `unnecessary_mut` (auto-fix)
- `unused_helper_functions` (auto-fix)
- `script_export_main` (auto-fix)
- `string_may_be_bare` (auto-fix)
- `inline_single_use_function` (auto-fix)
- `append_to_concat_assign` (auto-fix)

`posix` - Replace common bash/POSIX patterns.

- `ignore_over_dev_null` (auto-fix)
- `use_builtin_awk` (auto-fix)
- `use_builtin_bat` (auto-fix)
- `use_builtin_cat` (auto-fix)
- `use_builtin_date` (auto-fix)
- `use_sys_disks_instead_of_df` (auto-fix)
- `echo_just_identity` (auto-fix)
- `find_to_glob` (auto-fix)
- `use_sys_mem_instead_of_free` (auto-fix)
- `use_builtin_grep` (auto-fix)
- `head_to_first` (auto-fix)
- `use_sys_host_instead_of_hostname` (auto-fix)
- `use_builtin_cd` (auto-fix)
- `use_builtin_ls` (auto-fix)
- `use_builtin_pager` (auto-fix)
- `use_builtin_read` (auto-fix)
- `sed_to_str_replace` (auto-fix)
- `use_builtin_sort` (auto-fix)
- `use_builtin_tac` (auto-fix)
- `tail_to_last` (auto-fix)
- `use_sys_host_instead_of_uname` (auto-fix)
- `use_builtin_uniq` (auto-fix)
- `use_sys_host_instead_of_uptime` (auto-fix)
- `use_sys_users_instead_of_users` (auto-fix)
- `use_sys_users_instead_of_w` (auto-fix)
- `use_builtin_wc` (auto-fix)
- `use_sys_users_instead_of_who` (auto-fix)

`iteration` - Better patterns for loops and iteration.

- `replace_loop_counter_with_range`
- `replace_counter_while_with_each`

`runtime-errors` - Preventing unexpected runtime behaviour.

- `avoid_last_exit_code` (auto-fix)
- `check_complete_exit_code`
- `descriptive_error_messages`
- `escape_string_interpolation_operators`
- `exit_only_in_main`
- `check_typed_flag_before_use`
- `non_final_failure_check`
- `error_make_for_non_fatal` (auto-fix)
- `try_instead_of_do`
- `unsafe_dynamic_record_access` (auto-fix)
- `missing_stdin_in_shebang` (auto-fix)
- `dynamic_script_import`
- `catch_builtin_error_try`
- `unchecked_cell_path_index` (auto-fix)
- `unchecked_get_index` (auto-fix)
- `unchecked_first_last` (auto-fix)
- `wrap_external_with_complete`
- `use_over_source`
- `spread_list_to_external` (auto-fix)

`filtering` - Better patterns for filtering and selecting data.

- `each_if_to_where` (auto-fix)
- `for_filter_to_where`
- `omit_it_in_row_condition` (auto-fix)
- `slice_to_drop` (auto-fix)
- `slice_to_last` (auto-fix)
- `slice_to_skip` (auto-fix)
- `slice_to_take` (auto-fix)
- `where_closure_drop_parameter` (auto-fix)
- `remove_redundant_in` (auto-fix)

`performance` - Rules with potential performance impact

- `avoid_nu_subprocess`
- `dispatch_with_subcommands`
- `avoid_self_import`
- `turn_positional_into_stream_input` (auto-fix)
- `unnecessary_accumulate`
- `merge_multiline_print` (auto-fix)
- `chained_str_replace` (auto-fix)
- `streaming_hidden_by_complete` (auto-fix)

`type-safety` - Annotate with type hints where possible.

- `external_script_as_argument`
- `nothing_outside_function_signature` (auto-fix)
- `add_type_hints_arguments` (auto-fix)
- `string_param_as_path` (auto-fix)
- `missing_output_type` (auto-fix)
- `missing_in_type` (auto-fix)
- `avoid_nu_subprocess`
- `dynamic_script_import`

`documentation` - Improve actionability of user-facing messages.

- `add_doc_comment_exported_fn`
- `descriptive_error_messages`
- `add_label_to_error`
- `add_help_to_error`
- `add_span_to_label`
- `add_url_to_error`
- `main_positional_args_docs`
- `main_named_args_docs`
- `max_positional_params`
- `explicit_long_flags` (auto-fix)

`effects` - Handle built-in and external commands with side-effects.

- `dangerous_file_operations`
- `errors_to_stderr`
- `dont_mix_different_effects`
- `print_and_return_data`
- `each_nothing_to_for_loop` (auto-fix)
- `silence_stderr_data`

`external` - Replace common external CLI tools.

- `use_builtin_curl` (auto-fix)
- `use_builtin_fd` (auto-fix)
- `replace_jq_with_nu_get` (auto-fix)
- `use_builtin_wget` (auto-fix)
- `use_builtin_which` (auto-fix)

`formatting` - Formatting according to Nushell guidelines.

- `ansi_over_escape_codes` (auto-fix)
- `collapsible_if` (auto-fix)
- `forbid_excessive_nesting`
- `max_function_body_length`
- `replace_if_else_chain_with_match` (auto-fix)
- `block_brace_spacing` (auto-fix)
- `closure_brace_pipe_spacing` (auto-fix)
- `no_trailing_spaces` (auto-fix)
- `omit_list_commas` (auto-fix)
- `pipe_spacing` (auto-fix)
- `record_brace_spacing` (auto-fix)
- `reflow_wide_pipelines` (auto-fix)
- `reflow_wide_lists` (auto-fix)
- `wrap_wide_records` (auto-fix)

`naming` - Follow official naming conventions

- `kebab_case_commands`
- `screaming_snake_constants`
- `snake_case_variables` (auto-fix)
- `add_label_to_error`

`upstream` - Forward warnings and errors of the upstream Nushell parser.

- `dynamic_script_import`
- `nu_deprecated` (auto-fix)
- `nu_parse_error`

<!-- end-rule-groups -->

## Installation

### Recommended

The type installation that will always work on your system. From crates.io:

```bash
cargo install nu-lint
```

### Source

Build from source:

```bash
cargo install --path .
```

### Nix

Run without installing permanently (using flakes):

```bash
nix run git+https://codeberg.org/wvhulle/nu-lint
```

### Pre-Compiled Pinaries

Download the appropriate binary from the releases subpage.

## Editor extension

### VS Code extension

Available at [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=WillemVanhulle.nu-lint).

### Helix

Add to your `~/.config/helix/languages.toml`:

```toml
[language-server.nu-lint]
command = "nu-lint"
args = ["--lsp"]

[[language]]
name = "nu"
language-servers = ["nu-lint"]
```

The official Nu LSP server offers completion and some other hints. It should be configured out of the box for new Helix installations and environments that have Nushell installed.

### Neovim

Add to your Neovim configuration (Lua):

```lua
vim.lsp.config['nu-lint'] = {
  cmd = { 'nu-lint', '--lsp' },
  filetypes = { 'nu' },
  root_markers = { '.git' }
}
vim.lsp.enable('nu-lint')
```

You may want to configure official Nu language server in addition to this linter, see `nu --lsp` command.

### Emacs

Add to your Emacs configuration (with Eglot, built-in since Emacs 29):

```elisp
(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs
               '(nushell-mode "nu-lint" "--lsp")))
```

### Kate

Add to your `~/.config/kate/lspclient/settings.json`:

```json
{
  "servers": {
    "nushell": {
      "command": ["nu-lint", "--lsp"],
      "highlightingModeRegex": "^Nushell$"
    }
  }
}
```

### Other

You can also implement your own editor extensions using the `--lsp` flag as in: `nu-lint --lsp`. This will spawn a language server compliant with the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/).

## Configuration

Some rules are deactivated by default. Usually because they are too noisy or annoy people. You should activate them with the config file and a level override.

A configuration file is optional and should be named `.nu-lint.toml` in your project root. It may look like this:

```toml
# Some rules are configurable
max_pipeline_length = 80
pipeline_placement = "start"

# Set lint level of a set of rules at once.
[groups]
performance = "warning"
type-safety = "error"

# Override a single rule level
[rules]
dispatch_with_subcommands = "hint"
```

In this particular case, the user overrides the 'level' of certain groups and individual rules.

For any setting you don't set in that file, the defaults set in [./src/config.rs](./src/config.rs) will be used. If you specify the option in the configuration file, it will override the defaults.