# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
