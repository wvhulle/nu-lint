# Nu-Lint

Linter for the innovative [Nu](https://www.nushell.sh/) shell.

Learning to use a new shell is a radical change that can use some assistance. This project is aimed at helping new users of the [Nu](https://www.nushell.sh/) shell. Nu shell has a lot of interesting and useful features and this program will give you hints to use all the features of Nu.

Compatible with:

- The standard parser [nu-check](https://www.nushell.sh/commands/docs/nu-check.html).
- The standard formatter [topiary-nushell](https://github.com/blindFS/topiary-nushell).

## Installation

### Cross-platform

From crates.io:

```bash
cargo install nu-lint
```

### Source

Build from source:

```bash
cargo install --path .
cargo install --git . "$THIS_GIT_URL"
```

### Nix

To install in Nix or NixOS, add to `configuration.nix`:

```nix
let
  nu-lint = pkgs.callPackage (pkgs.fetchFromGitHub {
    owner = "wvhulle";
    repo = "nu-lint";
    rev = "COMMIT_HASH";
    sha256 = ""; # nix will tell you the correct hash
  }) {};
in
{
  environment.systemPackages = [
    nu-lint
  ];
}
```

Or in a `shell.nix`:

```nix
{ pkgs ? import <nixpkgs> {} }:

let
  nu-lint = pkgs.callPackage (pkgs.fetchFromGitHub {
    owner = "wvhulle";
    repo = "nu-lint";
    rev = "COMMIT_HASH";
    sha256 = ""; # nix will tell you the correct hash
  }) {};
in
pkgs.mkShell {
  buildInputs = [
    nu-lint
  ];
}
```

## Usage

Basic:

```bash
nu-lint                                    # Lint working directory
nu-lint script.nu                          # Lint a file
nu-lint directory/                         # Lint directory
```

```bash
nu-lint --config custom.toml script.nu     # Use custom config
nu-lint list-rules                         # Show all rules
nu-lint explain snake_case_variables       # Explain a rule
```

For LSP / editor plugins:

```bash
nu-lint --format json                      # Lint and output JSON
```

## Configuration

Create `.nu-lint.toml` in your project root (or any parent directory):

```toml
[general]
min_severity = "warning"

[rules]
snake_case_variables = "warning"
prefer_error_make = "info"
kebab_case_commands = "warning"
```

The linter will automatically find and use this config file when you run it.

## Rules

Categories:

- naming
- formatting
- idioms
- error handling
- code quality
- documentation
- type safety

## Example Output

```text
info[prefer_parse_over_each_split]

  ℹ Manual splitting with 'each' and 'split row' - consider using 'parse'
   ╭─[example.nu:5:1]
 5 │ $data | each { |line| $line | split row " " | get 0 }
   ·         ───────────────────┬──────────────────────────
   ·                            ╰── AST-based detection of structured text processing pattern
   ╰────
  help: Use 'parse "{field1} {field2}"' for structured text extraction instead of 'each' with 'split row'
```

This example demonstrates AST traversal detecting command patterns that the regex-based rules cannot catch.

## Planned features

Ideas for future improvements:

- Editor plugins
- Use external `jq` parser

## Contributing

Contributions are welcome. Please run tests and formatting before submitting:

```bash
cargo test
cargo +nightly fmt
cargo clippy --all-targets
cargo clippy --fix --allow-dirty --all-targets
```

### Running Benchmarks

Quick benchmark for performance testing:

```bash
cargo bench --bench prefer_builtin_rules prefer_builtin_small
```

This runs in ~5-10 seconds and measures AST traversal overhead for the prefer_builtin_* rules.

## License

MIT
