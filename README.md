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

Extra:

```bash
nu-lint --config custom.toml script.nu     # Use custom config
nu-lint list-rules                         # Show all rules
nu-lint explain snake_case_variables       # Explain a rule
```

For editor plugins (for LLMs / creators):

```bash
nu-lint --format json                      # Lint and output JSON
```

## Configuration

Create `.nu-lint.toml` in your project root (or any parent directory):

```toml
[general]
min_severity = "info"

[rules]
snake_case_variables = "warning"
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

## Planned features

Ideas for future improvements:

- Editor plugins such a VS Code extension
- Use external parsers for DSLs such as `jq`
- A lint plugin for Nu shell command line itself
- Better fix suggestions

## Contributing

Contributions are welcome.

Debugging

```bash
cargo test
```

Show debug output using the `instrument` function and an environment variable:

```bash
RUST_LOG=debug cargo test test_detect_unnecessary_variable_simple -- --nocapture
```

Please run linter and formatter before submitting PRs. Many optional and restrictive rules of Clippy have been turned on.

This will attempt to auto-fix violations in the Rust code.

```bash
cargo clippy --fix --allow-dirty --all-targets
```

Check if everything was fixed:

```bash
cargo clippy --all-targets
```

```
cargo +nightly fmt
```

Quick benchmark for performance testing:

```bash
cargo bench --bench prefer_builtin_rules prefer_builtin_small
```

## License

MIT
