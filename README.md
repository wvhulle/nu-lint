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

_(Editor integration is available, see below)_

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

- The official Nu parser [nu-check](https://www.nushell.sh/commands/docs/nu-check.html).
- The TreeSitter-based Nu formatter [topiary-nushell](https://github.com/blindFS/topiary-nushell).
- The official Nu [style guide](https://www.nushell.sh/book/style_guide.html)

Some of them need further testing and improvement. Please make an issue on the issue tracker to report any bugs. In early stages of development some rules may be replaced or renamed.

More than 100 rules are defined and most have automatic fixes available (list may be out-of-date):

`dead-code` - Remove unused or redundant code

- `avoid_self_import`
- `unnecessary_accumulate`
- `unnecessary_variable_before_return` (auto-fix)
- `redundant_ignore` (auto-fix)
- `unnecessary_mut` (auto-fix)
- `unused_helper_functions` (auto-fix)

`documentation` - Improve relevance of actionability of user-facing messages.

- `add_doc_comment_exported_fn`
- `descriptive_error_messages`
- `add_label_to_error`
- `add_help_to_error`
- `add_span_to_label`
- `add_url_to_error`
- `main_positional_args_docs`
- `main_named_args_docs`
- `max_positional_params`

`error-handling` - Error handling best practices

- `check_complete_exit_code`
- `descriptive_error_messages`
- `escape_string_interpolation_operators`
- `exit_only_in_main`
- `missing_stdin_in_shebang` (auto-fix)
- `non_final_failure_check`
- `use_error_make_for_catch` (auto-fix)
- `try_instead_of_do`
- `unsafe_dynamic_record_access` (auto-fix)

`external-tools` - Replace common external CLI tools.

- `use_builtin_curl` (auto-fix)
- `use_builtin_eza` (auto-fix)
- `use_builtin_fd` (auto-fix)
- `replace_jq_with_nu_get` (auto-fix)
- `use_builtin_rg` (auto-fix)
- `unnecessary_hat` (auto-fix)
- `use_builtin_wget` (auto-fix)
- `use_builtin_which` (auto-fix)

`formatting` - Formatting according to Nushell guidelines.

- `ansi_over_escape_codes` (auto-fix)
- `collapsible_if` (auto-fix)
- `forbid_excessive_nesting`
- `max_function_body_length`
- `replace_if_else_chain_with_match` (auto-fix)
- `brace_spacing` (auto-fix)
- `no_trailing_spaces` (auto-fix)
- `omit_list_commas` (auto-fix)
- `pipe_spacing` (auto-fix)
- `reflow_wide_pipelines` (auto-fix)
- `reflow_wide_lists` (auto-fix)
- `wrap_wide_records` (auto-fix)

`naming` - Follow official naming conventions

- `kebab_case_commands`
- `screaming_snake_constants`
- `snake_case_variables` (auto-fix)
- `add_label_to_error`

`performance` - Rules with potential performance impact

- `avoid_nu_subprocess`
- `dispatch_with_subcommands`
- `avoid_self_import`
- `turn_positional_into_stream_input` (auto-fix)
- `unnecessary_accumulate`
- `merge_multiline_print` (auto-fix)

`posix-tools` - Replace common bash/POSIX commands.

- `ignore_over_dev_null` (auto-fix)
- `use_builtin_awk` (auto-fix)
- `use_builtin_cat` (auto-fix)
- `use_builtin_cut` (auto-fix)
- `use_builtin_date` (auto-fix)
- `use_builtin_echo` (auto-fix)
- `use_builtin_find` (auto-fix)
- `use_builtin_grep` (auto-fix)
- `use_builtin_head` (auto-fix)
- `use_builtin_cd` (auto-fix)
- `use_builtin_ls` (auto-fix)
- `use_builtin_read` (auto-fix)
- `use_builtin_sed` (auto-fix)
- `use_builtin_sort` (auto-fix)
- `use_builtin_tail` (auto-fix)
- `use_builtin_uniq` (auto-fix)
- `use_builtin_wc` (auto-fix)
- `ansi_over_escape_codes` (auto-fix)

`side-effects` - Handle risky and unpredictable commands.

- `dangerous_file_operations`
- `errors_to_stderr`
- `avoid_mixed_io_types`
- `print_and_return_data`
- `silence_side_effect_only_each` (auto-fix)

`simplification` - Simplify verbose patterns to idiomatic Nushell

- `use_is_not_empty` (auto-fix)
- `dispatch_with_subcommands`
- `shorten_with_compound_assignment` (auto-fix)
- `lines_instead_of_split` (auto-fix)
- `lines_each_to_parse` (auto-fix)
- `simplify_regex_parse` (auto-fix)
- `split_row_first_last` (auto-fix)
- `split_row_get_to_parse` (auto-fix)
- `turn_positional_into_stream_input` (auto-fix)
- `replace_counter_while_with_each`
- `replace_loop_counter_with_range`
- `each_if_to_where` (auto-fix)
- `for_filter_to_where`
- `omit_it_in_row_condition` (auto-fix)
- `remove_redundant_in` (auto-fix)
- `where_or_filter_closure_to_it_row_condition` (auto-fix)
- `items_instead_of_transpose_each` (auto-fix)
- `merge_get_cell_path` (auto-fix)
- `merge_multiline_print` (auto-fix)
- `inline_single_use_function`

`type-safety` - Encourage annotations with type hints.

- `external_script_as_argument`
- `add_type_hints_arguments` (auto-fix)
- `prefer_path_type` (auto-fix)
- `typed_pipeline_io` (auto-fix)

`upstream` - Forward warnings and errors of the upstream Nushell parser.

- `nu_deprecated`
- `nu_parse_error`

## Installation

From crates.io:

```bash
cargo install nu-lint
```

### Source

Build from source:

```bash
cargo install --path .
```

### Nix

Run without installing (using flakes):

```bash
nix run git+https://codeberg.org/wvhulle/nu-lint
```

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

Create `.nu-lint.toml` in your project root:

```toml
# This rule is ignored
ignored = ["snake_case_variables"]

# Some rules are configurable
max_pipeline_length = 80
pipeline_placement = "start"

# Set lint level of a set of rules at once.
[groups]
performance = "warning"
type-safety = "error"

# Override a single rule lievel
[rules]
dispatch_with_subcommands = "hint"
```
