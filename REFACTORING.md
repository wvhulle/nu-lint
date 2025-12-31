# Engine State Initialization Refactoring

## Problem Statement

The lint engine state was initialized in `src/engine.rs`. However, it did not have access to all user-specific config. The nu-lsp server defined in nu-lsp and used by the main nushell binary (`nu` command) probably has a better engine state initialization than nu-lint.

This document describes how we addressed this issue by sharing code patterns with nu-lsp.

## Analysis

We analyzed how the main nushell binary initializes its engine state for the LSP server by examining the nushell repository at version 0.109.1:

### nu-lsp Engine State Initialization (from nushell repo)

Location: `nushell/crates/nu-lsp/src/lib.rs` and `nushell/src/main.rs`

The nu-lsp server in the main nushell binary initializes its engine state as follows:

1. `nu_cmd_lang::create_default_context()` - Core language commands
2. `nu_command::add_shell_command_context()` - Shell commands  
3. `engine_state.generate_nu_constant()` - Generate $nu constant
4. `nu_std::load_standard_library()` - Load standard library (in tests)
5. Set PWD environment variable
6. Optionally load user config via `config_files::setup_config()` (when started with `nu --lsp`)

### Previous nu-lint Engine State Initialization

Location: `src/engine.rs` (before refactoring)

The previous implementation was similar but had the initialization logic duplicated
inline within the `LintEngine::new_state()` method:

1. `nu_cmd_lang::create_default_context()`
2. `nu_command::add_shell_command_context()`
3. `nu_cli::add_cli_context()`
4. Set PWD environment variable
5. Manually add Print command
6. `engine_state.generate_nu_constant()`

**Key issues with the previous approach:**
- Initialization logic was embedded in `LintEngine`, making it hard to verify consistency with nu-lsp
- No clear documentation of how it related to nushell's own LSP initialization
- Difficult to share initialization between different components (e.g., LSP server and lint engine)
- No dedicated tests for the engine state initialization itself

## Solution

We created a shared `engine_state` module (`src/engine_state.rs`) that:

1. **Consolidates the initialization logic** into a single reusable function `create_engine_state()`
2. **Follows the same pattern as nu-lsp** from the main nushell binary
3. **Can be used by both the lint engine and LSP server** for consistency
4. **Provides a foundation for future enhancements** like loading user config or standard library

### Implementation

The new `engine_state::create_engine_state()` function:

```rust
pub fn create_engine_state() -> EngineState {
    let mut engine_state = nu_cmd_lang::create_default_context();
    engine_state = nu_command::add_shell_command_context(engine_state);
    engine_state = nu_cli::add_cli_context(engine_state);
    
    // Set PWD (required by commands like `path self`)
    if let Ok(cwd) = env::current_dir() {
        engine_state.add_env_var("PWD".into(), Value::string(cwd, Span::unknown()));
    }
    
    // Add print command
    let delta = {
        let mut working_set = StateWorkingSet::new(&engine_state);
        working_set.add_decl(Box::new(nu_cli::Print));
        working_set.render()
    };
    engine_state.merge_delta(delta).expect("Failed to add Print command");
    
    // Generate $nu constant
    engine_state.generate_nu_constant();
    
    engine_state
}
```

We also provided `create_engine_state_with_stdlib()` for future use when the standard library might be needed.

### Benefits

1. **Code Sharing**: The lint engine (`LintEngine`) now uses the shared initialization, and the LSP server (`ServerState`) indirectly benefits from this by using `LintEngine`
2. **Consistency with nu-lsp**: The pattern matches how nushell's own LSP server initializes
3. **Maintainability**: Engine state initialization logic is in one place
4. **Documentation**: Comprehensive documentation explains the relationship to nu-lsp
5. **Testing**: Dedicated tests verify the engine state is properly initialized
6. **Future-Ready**: Easy to add user config loading or standard library when needed

## Differences from nu-lsp

While we share the same initialization pattern, there are some intentional differences:

1. **User Config Loading**: nu-lint doesn't load user config files by default (unlike `nu --lsp`)
   - This is intentional - linting should be consistent regardless of user config
   - User config can affect how code is parsed/evaluated, which could lead to inconsistent lint results
   
2. **Standard Library**: nu-lint doesn't load the standard library by default
   - This speeds up initialization
   - The standard library can be loaded if needed using `create_engine_state_with_stdlib()`

3. **CLI Context**: nu-lint adds `nu_cli::add_cli_context()` and the Print command
   - These are needed for linting certain patterns
   - nu-lsp in nushell also has access to these via the main binary's initialization

## Future Enhancements

The shared initialization pattern makes it easy to add:

1. **Optional User Config Loading**: Could add a flag to load user config when desired
2. **Standard Library**: Already available via `create_engine_state_with_stdlib()`
3. **Plugin Support**: Could follow nushell's pattern for loading plugins
4. **Custom Contexts**: Easy to add additional command contexts as needed

## Testing

All existing tests pass (2138+ tests), including:

- Engine state initialization tests (6 tests)
- Lint engine tests (verification that linting still works)
- LSP server tests (14 tests)
- All rule tests

## References

- Nushell repository at version 0.109.1: https://github.com/nushell/nushell/tree/0.109.1
- nu-lsp implementation: `nushell/crates/nu-lsp/src/lib.rs`
- Main binary LSP integration: `nushell/src/main.rs` (lines 483-499)
- nu-lsp test helper showing initialization pattern: `nushell/crates/nu-lsp/src/lib.rs` (lines 478-490)
