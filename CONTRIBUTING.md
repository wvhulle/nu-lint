# Contributing

Contributions are welcome.

## Getting the anti-pattern clear

The idea of this project is to convert ideas about anti-patterns in Nu to AST traversal-based lint rules.

Before starting on a new rule, check if there is no existing rule that needs to be tweaked to cover this new anti-pattern. If not, you can start on a new rule.

Experiment with different variations of the anti-pattern. For example, you want to forbid `nothing` in non-signature positions. Here is a checklist:

- In which *contexts does is this an anti-pattern*. Just in signatures? Or also as the return type of a block? Can You use it literals?
- Is it possible to detect this reliably from AST? Can you use the information of Nu parser to give predictable detections of the anti-pattern? If not, you need strong justification to use heuristics, which should be avoided.
- Can this anti-pattern be fixed? When can it not be fixed?
- Should we offer different fixes depending on the situation? Preferably, you split a rule and anti-pattern in different sub-rules when the fixes are wide apart.

## Testing locally

Either you launch a Nushell with the `nu` command and try commands interactively or you specify commands in a string.

You can also create a test file `/tmp/test.nu`:

```nu
def main [] {
  SOME_ANTI_PATTERN
}
```

Then run it with `nu /tmp/test.nu` and observe the output in the terminal.

You can compare differences between shells by running `nu -c 'TEST_NU_CODE'` (for Nu test code)  and `bash -c 'TEST_BASH_CODE'` (for Bash test code).

- **Use descriptive IDs**: Rule IDs should describe what they detect (e.g., `string_param_as_path`), not the fix (avoid `prefer_X` or `use_Y`).

## Rule implementation structure

Each rule lives in its own directory under `src/rules/`. A typical rule consists of:

```text
src/rules/your_rule_name/
├── mod.rs           # Rule implementation
├── detect_bad.rs    # Tests for code that should trigger the rule
├── ignore_good.rs   # Tests for code that should NOT trigger the rule
```

Rules can also be grouped into submodules (e.g., `parsing/`, `filesystem/`, `typing/`) when they share common functionality.

## Implementing the DetectFix trait

Before you actually start implementing, you need to do some scaffolding. Don't start implementing yet, but this step is important to hook up your new rule in the linter and run its tests with the other rules.

Every rule implements the `DetectFix` trait from `src/rule.rs`:

```rust
use crate::{
    Fix, LintLevel, Replacement,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct YourRule;

impl DetectFix for YourRule {
    type FixInput<'a> = YourFixData; // Data needed for the fix, or () if no fix

    fn id(&self) -> &'static str {
        "your_rule_id"  // Unique identifier, used in config files, should be at least 3 descriptive words long with underscores and lower case
    }

    fn explanation(&self) -> &'static str {
        "Brief description of what this rule detects" // This will be displayed in the `--help` output and users should immediately know what this rule is doing in which situation in concise sentence.
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/relevant_command.html") // Optional, don't add if you don't find a good resource that documents this anti-pattern.
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning  // Or Hint, Error
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        // Detection logic - return violations with fix data
        context.detect_with_fix_data(|expr, ctx| {
            // Check each expression in the AST
            vec![]
        })
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        // Generate the fix using fix_data. This method is optional and should only be implemented if fix generation is reliable.
        Some(Fix::with_explanation(
            "Description of the fix",
            vec![Replacement::new(fix_data.span, "replacement text")],
        ))
    }
}
```

The associated type `FixData` is supposed to contain semantical data only (avoid strings), since that is the only reliable data that should be used to construct fixes.

You are not required to implement `Fix`, it has a default trait implementation.

After implementing `DetectFix` you should erase the associated type `FixData` by casting it to a dynamic dispatch object:

```rs
pub static RULE: &dyn Rule = &YourRule;
```

## Registering your rule

After implementing your rule, register it in two places:

1. **`src/rules/mod.rs`**: Add the module declaration and add your rule to `USED_RULES`
2. **`src/rules/groups.rs`**: Add your rule to the appropriate group(s)

Once your rule is added, you can run it as part of the whole test suite, but first you will want to run the test for this rule in particular with debug output (in case of failure):

```bash
cargo test UNIQUE_TEST_NAME -- --nocapture
```

## Test cases

Before actually starting to implement, you need to define tests. If you write tests after the implementation, you will often not catch edge cases. So it is important that you have a good picture of what is good and bad code and what you want to tell to the user. Do not duplicate tests just by renaming variables.

```text
src/rules/your_rule_name/
├── mod.rs           # Rule implementation
├── detect_bad.rs    # Tests for code that should trigger the rule
├── ignore_good.rs   # Tests for code that should NOT trigger the rule
└── generated_fix.rs # Tests for auto-fix output (optional but recommended)
```

## Parsing

Before you start implementing the detection itself, you need investigate how the upstream Nu parser (crate `nu-parser`) parses the fragment representing your new anti-pattern.

You can use this binary and the `--ast` flag:

```bash
cargo run -- --ast 'def main [] { if $in { ".md" } }'
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

The codebase provides several helper traits in `src/ast/`:

- `CallExt` (`src/ast/call.rs`): Methods for inspecting function calls
- `ExpressionExt` (`src/ast/expression.rs`): Methods for expression analysis
- `SpanExt` (`src/ast/span.rs`): Span manipulation utilities

Check existing rules for examples of common detection patterns.

## Debugging

You can debug the code in two ways:

- Use the built-in Rust macro `dbg!`
- Use a custom formatter for `env_logger`:
  - Add a call to the function `crate::log::init_log()` at the beginning of the test you are debugging
  - Call debug macros of the `log` crate in the new rule implementation.

For example, you could run one test named `test_detect_unnecessary_variable_simple`:

```bash
cargo test test_detect_unnecessary_variable_simple -- --nocapture
```

The output of the log macros will be shown together with relative file paths. You can use this to fix issues in the implementation.

From this moment, you need to repeat the previous steps over and over until you detect the anti-patterns you planned to detect. Only erase tests if they really don't belong to the category of mistakes you want to guard against.

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
