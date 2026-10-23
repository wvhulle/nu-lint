# Contributing

Contributions are welcome.

## Getting the anti-pattern clear

The idea of this project is to convert ideas about anti-patterns in Nu to AST traversal-based lint rules.

Before starting on a new rule, check if there is no existing rule that needs to be tweaked to cover this new anti-pattern. If not, you can start on a new rule.

Experiment with different variations of the anti-pattern. For example, you want to forbid `nothing` in non-signature positions. Here is a checklist:

- In which _contexts does is this an anti-pattern_. Just in signatures? Or also as the return type of a block? Can You use it literals?
- Is it possible to detect this reliably from AST? Can you use the information of Nu parser to give predictable detections of the anti-pattern? If not, you need strong justification to use heuristics, which should be avoided.
- Can this anti-pattern be fixed? When can it not be fixed?
- Should we offer different fixes depending on the situation? Preferably, you split a rule and anti-pattern in different sub-rules when the fixes are wide apart.

## Testing Locally

If you are a human, you should launch a Nushell with the `nu` command and try commands interactively. Otherwise, if you are an AI agent you should you should specify Nu test commands in a string and pass them to the interpreter directly.

One of the simplest options is to just create a test file `/tmp/test.nu` and use it to experiment. If will almost always look like this:

```nu
#!/usr/bin/env -S nu --stdin

def main [] {
  SOME_ANTI_PATTERN
}
```

Make it executable and run it with `/tmp/test.nu` and observe the output in the terminal.

You can compare differences between shells by running `nu -c 'TEST_NU_CODE'` (for Nu test code) and `bash -c 'TEST_BASH_CODE'` (for Bash test code).

## Crate Layout

The repository is a cargo workspace with three crates:

| Crate | Path | Contents |
|-------|------|----------|
| `nu-lint-analysis` | `nu-lint-analysis/` | AST helpers, `LintContext`, the `Rule` trait, violations, fixes and the lint engine |
| `nu-lint-rules` | `nu-lint-rules/` | Every lint rule, `USED_RULES` and the rule groups |
| `nu-lint` | repository root | Command line interface, LSP server, config loading and output formatting |

`cargo build` and `cargo test` at the repository root cover all three crates.

## Rule Implementation Structure

Each rule lives in its own directory under `nu-lint-rules/src/`. A typical rule consists of:

```text
nu-lint-rules/src/your_rule_name/
├── mod.rs           # Rule implementation
├── detect_bad.rs    # Tests for code that should trigger the rule
├── ignore_good.rs   # Tests for code that should NOT trigger the rule
```

A test file for fixes may be added when the false negative and false positive tests are working correctly (typically named generated_fix.rs, see later).

Rules can also be grouped into submodules (e.g., `parsing/`, `filesystem/`, `typing/`) when they share common functionality.

## Implementing the `DetectFix` Trait

Before you actually start implementing, you need to do some scaffolding. Don't start implementing yet, but this step is important to hook up your new rule in the linter and run its tests with the other rules.

Every rule implements the `DetectFix` trait from `nu-lint-analysis/src/rule.rs`.

You are not required to implement a `fix`, it has a default trait implementation.

After using the default `FixData = ()` (or coming up with your own) you should erase the associated type `FixData` by casting it to a dynamic dispatch object:

```rs
pub static RULE: &dyn Rule = &YourRule;
```

This allows the rule to appear in an array without heap allocation.

## Registering Your Rule

After implementing your rule, register it in two places:

1. **`nu-lint-rules/src/lib.rs`**: Add the module declaration and add your rule to `USED_RULES`
2. **`nu-lint-rules/src/groups.rs`**: Add your rule to the appropriate group(s)

Once your rule is added, you can run it as part of the whole test suite, but first you will want to run the test for this rule in particular with debug output (in case of failure):

```bash
cargo test UNIQUE_TEST_NAME -- --nocapture
```

## Test cases

Before actually starting to implement, you need to define tests. If you write tests after the implementation, you will often not catch edge cases. So it is important that you have a good picture of what is good and bad code and what you want to tell to the user. Do not duplicate tests just by renaming variables.

```text
nu-lint-rules/src/your_rule_name/
├── mod.rs           # Rule implementation
├── detect_bad.rs    # Tests for code that should trigger the rule
├── ignore_good.rs   # Tests for code that should NOT trigger the rule
└── generated_fix.rs # Tests for auto-fix output (optional but recommended)
```

Tests call the assertion methods on `dyn Rule` from `nu-lint-analysis/src/rule.rs` (enabled for other crates by the `test-support` feature), such as `RULE.assert_detects(code)`, `RULE.assert_ignores(code)` and `RULE.assert_count(code, n)`. For fixes, assert the complete output with `RULE.assert_fixed_is(bad_code, expected_code)`. Use the looser `assert_fixed_contains` only when the full output cannot be pinned down.

## Parsing

Before you start implementing the detection itself, you need investigate how the upstream Nu parser (crate `nu-parser`) parses the fragment representing your new anti-pattern.

You can use this binary and the `dev ast` command:

```bash
cargo run -- dev ast 'def main [] { if $in { ".md" } }'
```

Or from Bash, call the Nu interpreter to show the AST (note: this shows block IDs instead of expanded blocks):

```bash
nu -c 'ast --json benches/fixtures/small_file.nu | get block'
```

Alternatively, within a running Nu session, you can use Nu's built-in command to interactively explore a JSON version (you need an interactive Nu session open):

```bash
ast --json benches/fixtures/small_file.nu | get block | from json | explore
```

## Detection

The next step is implementing the detection of the anti-pattern.

In most cases you will want to use the Traverse trait with method `flat_map` on the `context.ast` object representing the abstract syntax tree parsed by the Nu parser.

```rs
let mut results = Vec::new();
let f = |expr: &Expression| collector(expr, self);
self.ast.flat_map(self.working_set, &f, &mut results);
results
```

The difficult part is finding an `f` that detects the expressions corresponding to your antipattern and ignores the good ones.

The codebase provides several helper traits in `nu-lint-analysis/src/ast/`:

- `CallExt` (`call.rs`): Methods for inspecting function calls
- `ExpressionExt` (`expression.rs`): Methods for expression analysis
- `SpanExt` (`span.rs`): Span manipulation utilities

Check existing rules for examples of common detection patterns.

Do not search, split or trim source text (`span_text`, `expr_text`, `source_before_span`, …) with string methods such as `contains`, `starts_with`, `find` or `trim`. Match on the AST nodes and spans instead, and add a helper to `nu-lint-analysis/src/ast/` when one is missing. The `source-text-search` ast-grep rule reports violations (see [Checks and entry points](#checks-and-entry-points)).

## Debugging

You can debug the code in two ways:

- Use the built-in Rust macro `dbg!`
- Use log output:
  - Add a call to the function `nu_lint_analysis::log::init_test_log()` at the beginning of the test you are debugging
  - Call debug macros of the `log` crate in the new rule implementation.

For example, you could run one test named `test_detect_unnecessary_variable_simple`:

```bash
cargo test test_detect_unnecessary_variable_simple -- --nocapture
```

The output of the log macros will be shown together with relative file paths. You can use this to fix issues in the implementation.

From this moment, you need to repeat the previous steps over and over until you detect the anti-patterns you planned to detect. Only erase tests if they really don't belong to the category of mistakes you want to guard against.

## Implementing auto-fix

The associated type `FixData` is supposed to contain semantical data only such as spans. Avoid strings, names or other types of plain-text, since that is not reliable data that should be used to construct fixes (strings may appear anywhere duplicated with different meanings).

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

Lint and fix timings on the fixtures in `benches/fixtures/`:

```bash
cargo bench --bench speed fixtures
```

Timing of each rule in isolation on the [nu_scripts](https://github.com/nushell/nu_scripts) corpus. The nix devshell sets `NU_SCRIPTS_DIR` to the pinned flake input; without it the group is skipped. `per_rule/parse_only` runs with every rule disabled and is the baseline to compare against:

```bash
cargo bench --bench speed per_rule
```

Comparative benchmarks against a baseline:

```bash
git checkout main
cargo bench --bench speed -- --save-baseline main
git checkout your-branch
cargo bench --bench speed -- --baseline main
```

## Releasing

Releases are automated with [release-plz](https://release-plz.dev) on Woodpecker CI (`.woodpecker/`):

1. Every push to `main` runs the tests and then opens or updates a release PR on Codeberg. That PR bumps the version from the conventional commit messages and updates `CHANGELOG.md`. Only `feat`, `fix` and `perf` commits trigger a release PR.
2. Merging the release PR publishes the crate to crates.io and creates the `v<version>` tag and Codeberg release.
3. The Codeberg release event builds the static musl binaries (`nix build .#dist`) and attaches them with sha256 checksums.

To preview the next version and changelog locally, run `nix develop -c release-plz update` and discard the result afterwards.

## Checks and entry points

Everything CI runs is a flake output, so it can be run locally in the same way:

| Command | Purpose |
|---------|---------|
| `nix flake check` | Tests and `cargo fmt` check |
| `sg scan` / `sg test` | Project-specific ast-grep rules in `.ast-grep/rules/` and their tests in `.ast-grep/rule-tests/` |
| `nix run .#check-commits -- main..@` | Conventional commit message check for a revision range |
| `nix build .#dist` | Static release binaries as `.tar.gz` |
| `nix run .#release` | Publish an unreleased version to crates.io and create its tag and Codeberg release (CI only, needs `CODEBERG_TOKEN`) |
| `nix run .#release-pr` | Open or update the release PR (CI only, needs `CODEBERG_TOKEN`) |

Git hooks are optional. Install them with [prek](https://github.com/j178/prek) (`prek install`) or [pre-commit](https://pre-commit.com) (`pre-commit install`) to run the commit message and formatting checks from `.pre-commit-config.yaml` locally.

## Cachix

Check if it is already in the cachix cache with something like:

```bash
nix-store --realise /nix/store/c0qksplq2v9i2a0ksvxfwp4npizjkmhg-nu-lint-0.0.121 --store https://wvhulle.cachix.org
```
