# Contributing

Contributions are welcome.

## Adding new lints

Always start from valid Nu shell code. Experiment with different versions of the same Nu fragment to find out the limitations and possibilities that Nu has compared to other shells.

You can compare by running: `nu -c 'TEST'`  and `bash -c 'TEST'`.

The first thing you might want to do for a new rule is investigate how Nu parses the fragment. Nu has a built-in command that allows you to interactively explore

```bash
ast --json benches/fixtures/small_file.nu | get block | from json | explore
```

If you are using Bash (or Claude is calling Bash), call the Nu interpreter from a Bash shell to show a JSON representation. This JSON representation is easier to read:

```bash
nu -c 'ast --json benches/fixtures/small_file.nu | get block'
```

Please follow a somewhat predictable test file structure. Currently the convention is to have three different test files for each new rule:

- `detect_bad.rs`: for the `assert_detects` assertions
- `ignore_good.rs`: for the `assert_ignores` assertions
- `generated_fix.rs`: for assertions about the contents of the replacement text or help text (optional if it is too hard to provide fixes, but recommended)

## Debugging

The primary way to debug new rules is:

- Add a call to the function `crate::log::instrument()` at the beginning of the test you are debugging
- Call debug macros of the `log` crate in the new rule implementation.
- Set the `RUST_LOG` environment variable.

For example, you could run one test named `test_detect_unnecessary_variable_simple` in any test file with the RUST_LOG environment variable:

```bash
RUST_LOG=debug cargo test test_detect_unnecessary_variable_simple -- --nocapture
```

The output of the log macros will be shown together with relative file paths. You can use this to fix issues in the implementation.

(It would be possible to use the [test-log](https://crates.io/crates/test-log) crate but I preferred implementing a custom formatter for `env-logger` displaying relative file links.)

## Linting

Please run linter and formatter before submitting PRs. Many optional and restrictive rules of Clippy have been turned on.

This will attempt to auto-fix violations in the Rust code.

```bash
cargo clippy --fix --allow-dirty --all-targets
```

```bash
cargo fmt
```

## Benchmarks

Quick benchmark for performance testing:

```bash
cargo bench --bench speed lint
```

Comparative benchmarks against a baseline:

```bash
git checkout main
cargo bench --bench speed -- --save-baseline main
git checkout your-branch
cargo bench --bench speed -- --baseline main
```
