# Nu-Lint

Linter for the innovative [Nu](https://www.nushell.sh/) shell.

Learning to use a new shell is a radical change that can use some assistance. This project is aimed at helping new and intermediate users of the [Nu](https://www.nushell.sh/) shell. Nu shell has a lot of useful features not found in other scripting languages. This linter will give you hints to use all of them and even offer automatic fixes.

All rules are optional and can be disabled with a configuration file. The rule definitions are designed to be compatible with:

- The standard Nu parser [nu-check](https://www.nushell.sh/commands/docs/nu-check.html).
- The standard Nu formatter [topiary-nushell](https://github.com/blindFS/topiary-nushell).

## Example

The rule `positional_to_pipeline` recommends to use pipelines instead of positional arguments:

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

## CLI usage

For all available options and usage information, run:

```bash
nu-lint # Lint all Nu files in working directory
nu-lint --help
```

More than 90 rules are available on The CLI. Some of them need further testing and improvement. Please make an issue on the issue tracker to report any bugs.

Here is a snapshots of the rules (may be slightly out-of-date, see git for latest):

- Documentation quality rules
  - add_doc_comment_exported_fn
  - descriptive_error_messages
  - add_label_to_error
  - add_help_to_error
  - add_span_to_label
  - add_url_to_error
  - main_positional_args_docs
  - main_named_args_docs
  - max_positional_params

- Error handling best practices
  - add_label_to_error
  - add_help_to_error
  - add_span_to_label
  - add_url_to_error
  - check_complete_exit_code
  - descriptive_error_messages
  - escape_string_interpolation_operators
  - exit_only_in_main
  - missing_stdin_in_shebang
  - non_final_failure_check
  - make_error_from_exit
  - try_instead_of_do
  - errors_to_stderr
  - unsafe_dynamic_record_access

- Replace modern CLI tools with native Nushell equivalents
  - use_builtin_curl
  - use_builtin_eza
  - use_builtin_fd
  - replace_jq_with_nu_get
  - use_builtin_rg
  - unnecessary_hat
  - use_builtin_wget
  - use_builtin_which

- Check that code is formatted according to the official Nushell guidelines.
  - collapsible_if
  - forbid_excessive_nesting
  - max_function_body_length
  - replace_if_else_chain_with_match
  - brace_spacing
  - no_trailing_spaces
  - omit_list_commas
  - pipe_spacing
  - reflow_wide_pipelines
  - reflow_wide_lists
  - wrap_wide_records

- Linting rules for naming conventions
  - kebab_case_commands
  - screaming_snake_constants
  - snake_case_variables

- Performance optimization hints
  - avoid_self_import
  - avoid_nu_subprocess
  - use_builtin_is_not_empty
  - dispatch_with_subcommands
  - shorten_with_compound_assignment
  - unnecessary_accumulate
  - lines_instead_of_split
  - parse_instead_of_split
  - turn_positional_into_stream_input
  - while_counter
  - loop_counter
  - where_instead_each_then_if
  - filter_collect_with_where
  - remove_redundant_in
  - row_condition_above_closure
  - unnecessary_variable_before_return
  - inline_single_use_function
  - items_instead_of_transpose_each
  - merge_get_cell_path
  - merge_multiline_print
  - redundant_ignore
  - unnecessary_mut
  - unused_helper_functions

- Replace common bash/POSIX commands with native Nushell equivalents
  - ignore_over_dev_null
  - use_builtin_awk
  - use_builtin_cat
  - use_builtin_cut
  - use_builtin_date
  - use_builtin_echo
  - use_builtin_find
  - use_builtin_grep
  - use_builtin_head
  - use_builtin_cd
  - use_builtin_ls
  - use_builtin_read
  - use_builtin_sed
  - use_builtin_sort
  - use_builtin_tail
  - use_builtin_uniq
  - use_builtin_wc

- Commands that escape the type system
  - dangerous_file_operations
  - mixed_io_types
  - print_and_return_data

- Rules for systemd service scripts
  - add_journal_prefix
  - attach_loglevel_to_log_statement

- Enforce explicit typing of variables and pipelines.
  - external_script_as_argument
  - missing_type_annotation
  - prefer_path_type
  - typed_pipeline_io
  - avoid_nu_subprocess

- Rules that detect issues also flagged by the Nushell parser.
  - nu_deprecated
  - nu_parse_error

## Installation

From crates.io:

```bash
cargo install nu-lint
```

### Source

Build from source:

```bash
cargo install --path .
cargo install --git "$THIS_GIT_URL"
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

# Set lint level of a set of rules at once.
[groups]
performance = "warning"
type-safety = "error"

# Override a single rule lievel
[rules]
prefer_pipeline_input = "hint"
```

## License

MIT
