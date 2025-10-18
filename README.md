# nu-lint

A *static analysis tool for [Nushell](https://www.nushell.sh/) scripts* **based on the official style guide**.

Can *detect stylistic issues in Nushell scripts* and suggest improvements. Functions complementary to the basic checks done by [nu-check](https://www.nushell.sh/commands/docs/nu-check.html).

(Inspired by Rust's Clippy linter tool.)

## Installation

From crates.io:

```bash
cargo install nu-lint
```

Or build from source:

```bash
cargo install --path .
```

## Usage

```bash
nu-lint script.nu                          # Lint a file
nu-lint directory/                         # Lint all .nu files in a directory
nu-lint --config custom.toml script.nu     # Use custom config
nu-lint list-rules                         # Show all rules
nu-lint explain snake_case_variables       # Explain a rule
```

## Configuration

Create `.nu-lint.toml` in your project root (or any parent directory):

```toml
[general]
max_severity = "warning"

[rules]
snake_case_variables = "warning"
prefer_error_make = "info"
kebab_case_commands = "warning"

[style]
line_length = 100
indent_spaces = 4

[exclude]
patterns = ["*.test.nu", "vendor/**"]
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

## Contributing

Contributions are welcome. Please run tests and formatting before submitting:

```bash
cargo test
cargo +nightly fmt
cargo clippy
```

## License

MIT
