# Copilot Instructions for nu-lint

Do not use bash-style redirections. You should use Nushell syntax.

Use nu shell syntax when generating shell commands.

Don't create summary documents.

## Testing Rules with AST Traversal

When writing tests for rules that use AST traversal (implementing `AstVisitor`), use the `LintContext::test_with_parsed_source` helper which automatically loads the Nushell standard library commands. Otherwise, commands like `each`, `split row`, `parse`, etc., will be parsed as external commands rather than proper Nushell calls.

### Correct Way to Write Tests

```rust
#[test]
fn test_my_rule() {
    let rule = MyRule::new();
    let code = r#"$items | each { |x| $x | split row " " }"#;

    LintContext::test_with_parsed_source(code, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());
    });
}
```

### ❌ Wrong Way (Don't Do This)

```rust
// This will NOT work for AST-based rules!
let engine_state = EngineState::new(); // Missing stdlib commands!
let (block, working_set) = parse_source(&engine_state, code.as_bytes());
```

### Why This Matters

- `EngineState::new()` creates a bare engine without any command definitions
- Without stdlib commands, the parser treats commands like `each`, `split row`, `where`, etc. as `ExternalCall` expressions with `GlobPattern` arguments
- Your AST visitor looking for `Expr::Call` will never match, causing tests to fail
- The pattern `^git` explicitly creates external calls, but bare commands should be internal

### Pattern for All AST-Based Rules

1. Use `LintContext::test_with_parsed_source` for **every** test that parses Nushell code
2. This helper automatically provides an `EngineState` with stdlib commands loaded
3. The closure receives a fully configured `LintContext` ready for testing

### Example Rules Using This Pattern

- `src/rules/prefer_compound_assignment/` - Uses AST traversal for detecting compound assignment opportunities
- `src/rules/prefer_parse_over_each_split/` - Uses AST traversal for detecting `each` with `split row`

## AST Traversal vs Regex

### When to Use AST Traversal

Use AST traversal (implementing `AstVisitor`) when:

- You need to detect specific command calls (e.g., `each`, `split row`)
- You need to inspect closure/block contents
- You need semantic understanding of the code structure
- You need to avoid false positives from strings/comments

### When to Use Regex

Use regex patterns when:

- Checking naming conventions (kebab-case, snake_case, etc.)
- Looking for spacing/formatting issues
- The pattern is truly text-based and doesn't need semantic understanding

## Performance Consideration

Creating an `EngineState` with stdlib is expensive. In production code (`src/engine.rs`), we use `OnceLock` to cache it. In tests, `LintContext::test_with_parsed_source` creates it per test for isolation.

## Refactoring Large Rules

When a rule grows too large and covers too many concerns, split it into focused rules:

### When to Split

- Rule has 20+ command mappings or patterns
- Rule mixes different categories of commands (file operations, text processing, system commands)
- Description becomes too generic or needs long lists
- Tests become difficult to maintain

### How to Split

1. **Group by logical categories** - e.g., file operations, text transformation, system commands
2. **Keep common operations together** - Most frequently used commands in one rule
3. **Update rule naming** - Be specific about what each rule checks
4. **Update descriptions** - Clear, concise descriptions of scope
5. **Maintain test coverage** - Each new rule needs its own test file
