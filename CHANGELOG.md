# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.9.0](https://codeberg.org/wvhulle/nu-lint/compare/v1.8.0...v1.9.0) - 2026-11-05

### Added

- *(release)* attach macOS binaries to releases
- *(config)* keep accepting former rule ids

### Build

- *(ast-grep)* make every rule an error and add rule tests
- *(hooks)* hide convco repository warnings in the commit message hook
- *(ast-grep)* check generic fix structs in fix-data-plain-text
- *(ast-grep)* check optional fix structs in fix-data-plain-text
- *(ast-grep)* enforce the abstraction level of rules

### CI

- upload to cachix in separate steps
- open a weekly pull request updating dependencies
- separate dependency builds and quiet nix output in logs
- run formatting and ast-grep checks through prek

### Changed

- *(rules)* stop trimming operand text when building fixes
- *(rules)* replace source text checks with spans and variable ids
- *(rules)* keep source spans instead of command text in fix data
- *(rules)* derive parameter and variable names from the AST in fixes
- *(rules)* keep AST nodes instead of decoded text in fix data
- *(rules)* keep spans and AST nodes in optional fix data
- *(rules)* remove doc comments added to fix data
- *(snake_case_variables)* read the name through the variable helper
- *(rules)* use descriptive names for identifiers added to rules
- *(rules)* move renamed rules to directories matching their ids
- *(split_row_get_inline)* keep the split call instead of delimiter text
- *(rules)* read input and output annotations from the signature
- *(analysis)* put all signature layout behind one type
- *(compound_assignment)* compare variables by id
- *(record_assignments)* identify `$env` by its variable id
- *(error_make)* compare only literal record keys
- *(list_param_to_variadic)* label spans from the variable and signature layout
- *(rules)* share the implicit row condition parameter check
- *(not_is_empty_to_is_not_empty)* stop trimming fix text
- *(effects)* read literal argument values before source text
- *(analysis)* move comment inspection next to source lines

### Documentation

- *(rules)* use backticks for inline code in rule descriptions
- *(rules)* restore details dropped from rule descriptions

### Fixed

- *(redundant_echo)* ignore attributes that alias echo
- *(deps)* update nushell crates and lockfiles
- *(missing_stdin_in_shebang)* ignore arrows in parameter comments
- *(positional_to_pipeline)* find the parameter list and pipeline head from the AST
- *(where_closure_drop_parameter)* detect closures with spaces inside the pipes
- *(split_row_get_multistatement)* count fields from the string value
- *(custom_log_command)* recognize `use std log` as a stdlib import
- *(redundant_nu_subprocess)* detect quoted command flags
- *(filesystem)* read the raw flag of `open` from the call
- *(positional_to_pipeline)* offer no fix when `$in` would change meaning
- *(rules)* rename rule ids that were vague or too long
- *(split_row_get_multistatement)* stop deleting code in the fix
- *(unused_variable)* keep values with side effects
- *(positional_to_pipeline)* remove only the converted parameter
- *(explicit_long_flags)* ignore grouped short flags
- *(missing_output_type)* do not guess output from an unknown input
- *(each_nothing_to_for_loop)* produce a valid `for` statement
- *(positional_to_pipeline)* offer no fix for annotated signatures
- *(remove_hat_not_builtin)* keep the hat when a command shadows the name
- *(rules)* offer no parse fix for `split row` with flags
- *(add_type_hints_arguments)* infer parameter types from the compared value
- *(snake_case_variables)* replace each usage of a cell path head once
- *(unused_variable)* remove only the declaration inside nested blocks
- *(positional_to_pipeline)* offer no fix when the command has callers
- *(single_call_command)* do not inline multi-statement bodies
- *(unused_helper_functions)* count completers of `extern` signatures as usages
- *(missing_in_type)* only count `$in` at the head of a pipeline as input
- *(string_may_be_bare)* keep quotes where a bare word is invalid
- *(rules)* parenthesize command values in generated records
- *(error_make_for_non_fatal)* only quote bare word messages
- *(if_else_chain_to_match)* only rewrite equality with literals
- *(string_param_as_path)* replace only the type annotation
- *(add_type_hints_arguments)* do not take field arithmetic as the parameter type
- *(rules)* keep the written parameter list when adding type annotations
- *(if_else_chain_to_match)* match on the compared expression
- *(rules)* offer no record fix when a value reads an earlier assignment
- *(add_type_hints_arguments)* do not guess `int` from arithmetic with other values
- *(list_param_to_variadic)* rewrite only the converted parameter
- *(unused_parameter)* remove only the unused parameter
- *(unused_parameter)* keep the closing bracket on its own line
- *(transpose_items)* compare field names from the cell path
- *(unnecessary_accumulate)* compare the loop variable by id
- *(if_else_chain_to_match)* compare operands by variable and cell path
- *(add_type_hints_arguments)* do not guess `int` from arithmetic between variables
- *(rules)* infer types only where the operator proves them
- *(rules)* compare loop and counter variables by id
- *(dispatch_with_subcommands)* match the parameter by id
- *(append_to_concat_assign)* compare the whole cell path
- *(rules)* identify external commands by their parsed value
- *(rules)* read numeric arguments from the parsed value
- *(lines_instead_of_split)* compare the string value
- *(self_import)* find current file references in the AST
- *(filesystem)* read file extensions from literal paths only
- *(unused_parameter)* remove call arguments by argument spans

### Testing

- *(rules)* assert the complete fix output

## [1.8.0](https://codeberg.org/wvhulle/nu-lint/compare/v1.7.1...v1.8.0) - 2026-11-02

### Added

- *(get_optional_to_cell_path)* add rule replacing get -o with optional cell paths
- *(where_not_null_to_compact)* add rule replacing null filters with compact
- *(where_not_empty_to_compact)* add hint for empty string filters
- *(datetime_epoch_arithmetic)* add hint for time arithmetic on integers
- *(manual_duration_decomposition)* add hint for splitting time units by hand
- *(translate-bash-syntax)* add rules repairing pasted shell code

### CI

- record and publish recordings on woodpecker
- *(release)* set an upstream for main so release-plz can update its PR

### Changed

- *(analysis)* add CallExt::builtin_name to identify builtin and keyword calls
- *(translate-bash-syntax)* follow contributing guidelines

### Documentation

- generate webp in ci

### Fixed

- stop matching user-defined commands that share a builtin name
- *(merge_get_cell_path)* detect inside blocks and keep optional access in fixes
- *(division_to_format_duration)* remove auto-fix that changed the result type
- *(do_not_compare_booleans)* ignore optional cell paths that may be null
- *(missing_stdin_in_shebang)* detect pipeline input inside try and if blocks
- *(missing_output_type)* include nothing for an if without else
- *(analysis)* resolve builtin names in shared AST helpers
- *(effect)* look up builtin side effects from the resolved call
- resolve builtin names in remaining rules
- use utf16 for column calculations
- *(lsp)* keep non-conflicting edits in fix-all
- *(xtask)* track the recordings bookmark before pushing it
- *(ci)* fetch origin/main before setting it as the upstream of main

### Other

- remove useless join wrapped lines
- delete join_wrapped_lines source files

## [1.7.1](https://codeberg.org/wvhulle/nu-lint/compare/v1.7.0...v1.7.1) - 2026-10-25

### CI

- keep configuration docs updated through bot PR

### Fixed

- *(add_doc_comment_exported_fn)* use attribute block span or call's
- *(lsp)* skip file logging instead of panicking when log directory cannot be created

### Miscellaneous

- *(nu-lint)* release v1.7.0
- *(deps)* update cfg_aliases to 0.2.2 and sync xtask version in lockfile

## [1.7.0](https://codeberg.org/wvhulle/nu-lint/compare/v1.6.0...v1.7.0) - 2026-10-23

### Added

- more migrations of deprecated commands

### Build

- make the root manifest a cargo workspace

### CI

- slim the test toolchain and stop uploading test artifacts
- upload release binaries with a flake app instead of the tag-only plugin
- upload release assets with tea
- push final build outputs that cachix watch-exec misses
- enforce project ast-grep rules and replace text search rule with structural check

### Changed

- separate config data from config loading and validation
- pass the rule catalog into the lint engine and move file collection out
- move fix report formatting to the format module
- move the lsp log setup into the lsp module
- expose rule test helpers behind a test-support feature
- extract nu-lint-analysis crate with the AST, rule trait and lint engine
- extract nu-lint-rules crate
- import from nu-lint-analysis and nu-lint-rules directly instead of crate facades

### Documentation

- describe the workspace crates and run all of them by default

### Fixed

- build user group
- *(nothing_outside_signature)* stop hanging on calls with extra positional arguments
- build without the lsp feature by owning the diagnostic tag type
- silent skip of malformed config in lsp
- close lsp properly on shutdown

### Miscellaneous

- tidy repo root and move ast-grep rules into .ast-grep

### Other

- expose project as https://pre-commit.com/ hook

### Performance

- benchmark every rule on the nu_scripts corpus and all fixtures

### Changed

- **Breaking:** every rule now belongs to exactly one group, and the groups have descriptive names. Unknown group names in the config file are now an error instead of being ignored. Rename your `[groups]` keys:

  | Old group | New group |
  |---|---|
  | `runtime-errors` | `catch-runtime-failures` |
  | `effects` | `isolate-side-effects` |
  | `type-safety` | `annotate-types` |
  | `idioms` | `adopt-nu-syntax` |
  | `filtering`, `iteration` | `pipe-instead-of-loop` |
  | `parsing`, `filesystem` | `parse-structured-data` (`string_param_as_path` moved to `annotate-types`) |
  | `external` | `replace-external-tools` |
  | `performance` | `stream-in-process` |
  | `dead-code` | `remove-dead-code` |
  | `formatting` | `format-whitespace` (structural rules moved to the new `reduce-complexity`) |
  | `naming` | `follow-naming-conventions` |
  | `documentation` | `document-interfaces` |
  | `upstream` | `forward-parser-diagnostics` |
  | `migrations` | `migrate-deprecated-syntax` |

## [1.6.0](https://codeberg.org/wvhulle/nu-lint/compare/v1.5.0...v1.6.0) - 2026-10-16

### Added

- *(nu_deprecated)* autofix str upcase/downcase to str uppercase/lowercase
- *(filtering)* add uniq_to_set_operation for union, intersect and difference
- add nu_script_to_run hint for the run keyword
- add legacy_completer_inputs for the 0.116 completer interface

### Build

- *(deps)* bump nushell crates to 0.116.0

### CI

- replace release scripts with release-plz and fix woodpecker config
- split workflows into specifically named steps
- cache dependency builds and test in the dev profile

### Changed

- align external tool rules with contribution guidelines
- replace git-hooks.nix with a plain pre-commit config and flake entry points

### Documentation

- add TOC

### Fixed

- *(nu_deprecated)* remove doubled period between upstream label and help
- *(ci)* run release-plz in a devshell without git hooks
