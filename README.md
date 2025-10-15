# nu-lint

A linter for Nushell scripts.

## Features

- Fast static analysis using Nushell's AST parser
- Helpful diagnostics powered by miette
- Built-in rules covering style, best practices, performance, documentation, and type safety
- Configurable via `.nu-lint.toml`
- Extensible rule system

## Installation

```bash
cargo install --path .
```

Or run without installing:

```bash
cargo run --release -- path/to/script.nu
```

## Usage

```bash
nu-lint script.nu                          # Lint a file
nu-lint directory/                         # Lint all .nu files in a directory
nu-lint --config custom.toml script.nu     # Use custom config
nu-lint list-rules                         # Show all rules
nu-lint explain S001                       # Explain a rule
```

## Configuration

Create `.nu-lint.toml` in your project root (or any parent directory):

```toml
[general]
max_severity = "warning"

[rules]
S001 = "warning"  # snake-case-variables
BP001 = "info"    # prefer-error-make

[style]
line_length = 100
indent_spaces = 4

[exclude]
patterns = ["*.test.nu", "vendor/**"]
```

The linter will automatically find and use this config file when you run it.

## Rules

View all available rules:

```bash
nu-lint list-rules
```

Get details about a specific rule:

```bash
nu-lint explain <RULE_ID>
```

## Example output

```text
warning(S001)

  ⚠ Variable 'myVariable' should use snake_case naming convention
   ╭─[3:5]
 3 │ let myVariable = 5
   ·     ─────┬────
   ·          ╰── Variable 'myVariable' should use snake_case naming convention
   ╰────
  help: Consider renaming to: my_variable
```

## Development

```bash
cargo build --release              # Build optimized binary
cargo test                         # Run all tests
cargo clippy                       # Check for lint warnings
```

## License

MIT
