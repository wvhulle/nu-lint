# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
