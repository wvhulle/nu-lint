# Contributing

Contributions are welcome.

## Ideas

Ideas for future improvements:

- Editor plugins such a VS Code extension
- Use external parsers for DSLs such as `jq`
- A lint plugin for Nu shell command line itself
- Better fix suggestions

Maybe useful for people who want to create editor integration:

```bash
nu-lint --format json                      # Lint and output JSON
```

## Adding new lints

Use Nu shells AST command.

```bash
ast --json update-vscode-settings.nu | get block | from json | explore
```

## Debugging

Use `lldb` extension to step through code.

Show debug output using the `instrument` function and an environment variable:

```bash
RUST_LOG=debug cargo test test_detect_unnecessary_variable_simple -- --nocapture
```

(It would be possible to use the [test-log](https://crates.io/crates/test-log) crate but I prefered a custom formatter displaying file links.)

## Testing

Debugging is primarily done with the tests.

```bash
cargo test
```

Tests follow a pattern where each rule has 'detect', 'fix' and 'ignore' tests.

## Linting

Please run linter and formatter before submitting PRs. Many optional and restrictive rules of Clippy have been turned on.

This will attempt to auto-fix violations in the Rust code.

```bash
cargo clippy --fix --allow-dirty --all-targets
```

Check if everything was fixed:

```bash
cargo clippy --all-targets
```

```bash
cargo +nightly fmt
```

## Benchmarks

Quick benchmark for performance testing:

```bash
cargo bench --bench lint_performance lint_small_file
```

Comparative benchmarks

```bash
git checkout main
cargo bench --bench lint_performance lint_with_violations -- --save-baseline main
git checkout branch
cargo bench --bench lint_performance lint_with_violations -- --baseline main
```
