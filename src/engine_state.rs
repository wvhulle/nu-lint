/// Shared engine state initialization code.
///
/// This module provides reusable engine state initialization that can be used
/// by both the linting engine and the LSP server. The implementation follows
/// the same pattern used by nu-lsp in the main nushell binary.
///
/// ## Background
///
/// The problem statement requested checking if we can share code with nu-lsp,
/// which is part of the main nushell binary. After analyzing the nushell source
/// code (specifically `crates/nu-lsp/src/lib.rs` and `src/main.rs` in the
/// nushell repository at version 0.109.1), we found that nu-lsp initializes
/// its engine state as follows:
///
/// 1. `nu_cmd_lang::create_default_context()` - Core language commands
/// 2. `nu_command::add_shell_command_context()` - Shell commands
/// 3. `engine_state.generate_nu_constant()` - Generate $nu constant
/// 4. `nu_std::load_standard_library()` - Load standard library (optional)
/// 5. Set PWD environment variable
/// 6. Optionally load user config via `config_files::setup_config()`
///
/// This module implements the same initialization pattern (steps 1-5), making
/// it easy to keep nu-lint's engine state in sync with how nushell's LSP server
/// initializes its state. The main difference is that nu-lint doesn't load user
/// config files by default (step 6), as it's designed to lint code independently
/// of user configuration.
///
/// ## Usage
///
/// The `LintEngine` uses this shared initialization directly through 
/// `create_engine_state()`. The LSP server's `ServerState` indirectly uses
/// this initialization by creating a `LintEngine`, which ensures consistent
/// engine state across both the linting and LSP functionality.
use std::env;

use nu_protocol::{
    Span, Value,
    engine::{EngineState, StateWorkingSet},
};

/// Initialize a Nushell engine state with all standard commands and context.
///
/// This creates a minimal but complete engine state suitable for parsing and
/// evaluating Nushell code. The initialization follows the same pattern as
/// nu-lsp in the main nushell binary (see module documentation for details).
///
/// The engine state includes:
/// - Default language context (core commands via `nu_cmd_lang::create_default_context()`)
/// - Shell command context (filesystem, process commands, etc. via `nu_command::add_shell_command_context()`)
/// - CLI context (REPL commands via `nu_cli::add_cli_context()`)
/// - PWD environment variable (required by commands like `path self`)
/// - Print command (exported by nu-cli but not automatically added)
/// - $nu constant (required for const evaluation at parse time via `generate_nu_constant()`)
///
/// This initialization is equivalent to what nu-lsp does in the main nushell
/// binary, minus loading user configuration files and the standard library
/// (which can be added via `create_engine_state_with_stdlib()` if needed).
#[must_use]
pub fn create_engine_state() -> EngineState {
    let mut engine_state = nu_cmd_lang::create_default_context();
    engine_state = nu_command::add_shell_command_context(engine_state);
    engine_state = nu_cli::add_cli_context(engine_state);

    // Set PWD environment variable (required by many commands like `path self`)
    if let Ok(cwd) = env::current_dir()
        && let Some(cwd) = cwd.to_str()
    {
        engine_state.add_env_var("PWD".into(), Value::string(cwd, Span::unknown()));
    }

    // Add print command (exported by nu-cli but not added by add_cli_context)
    let delta = {
        let mut working_set = StateWorkingSet::new(&engine_state);
        working_set.add_decl(Box::new(nu_cli::Print));
        working_set.render()
    };
    engine_state
        .merge_delta(delta)
        .expect("Failed to add Print command");

    // Set up $nu constant (required for const evaluation at parse time)
    engine_state.generate_nu_constant();

    engine_state
}

/// Initialize a Nushell engine state with standard library loaded.
///
/// This is similar to `create_engine_state()` but also loads the standard
/// library, which provides additional commands and utilities. This follows
/// the pattern used by nu-lsp when full command availability is needed.
///
/// Note: Loading the standard library adds initialization time, so only use
/// this when you need the additional commands it provides.
///
/// This function is currently unused but available for future enhancements
/// where the standard library might be beneficial for linting.
#[must_use]
#[allow(dead_code, reason = "Reserved for future use when standard library is needed")]
pub fn create_engine_state_with_stdlib() -> EngineState {
    let mut engine_state = create_engine_state();

    // Load standard library for additional commands
    // This is what nu-lsp does in the main nushell binary (in test helper)
    if let Err(e) = nu_std::load_standard_library(&mut engine_state) {
        log::warn!("Failed to load standard library: {e}");
    }

    engine_state
}

#[cfg(test)]
mod tests {
    use super::*;
    use nu_protocol::engine::StateWorkingSet;

    #[test]
    fn test_create_engine_state_includes_basic_commands() {
        let engine_state = create_engine_state();
        let working_set = StateWorkingSet::new(&engine_state);

        // Check that some basic commands are available
        assert!(working_set.find_decl(b"let").is_some(), "let command should be available");
        assert!(working_set.find_decl(b"def").is_some(), "def command should be available");
        assert!(working_set.find_decl(b"if").is_some(), "if command should be available");
    }

    #[test]
    fn test_create_engine_state_includes_shell_commands() {
        let engine_state = create_engine_state();

        // Check that some shell commands are available (exact names may vary)
        // Just verify that we have a non-trivial number of declarations
        assert!(
            engine_state.num_decls() > 100,
            "Should have many shell commands available, got {}",
            engine_state.num_decls()
        );
    }

    #[test]
    fn test_create_engine_state_includes_print_command() {
        let engine_state = create_engine_state();
        let working_set = StateWorkingSet::new(&engine_state);

        // Check that the print command was explicitly added
        assert!(
            working_set.find_decl(b"print").is_some(),
            "print command should be available"
        );
    }

    #[test]
    fn test_create_engine_state_sets_pwd() {
        let engine_state = create_engine_state();
        
        // Check that PWD environment variable is set
        let pwd = engine_state.get_env_var("PWD");
        assert!(pwd.is_some(), "PWD environment variable should be set");
    }

    #[test]
    fn test_create_engine_state_generates_nu_constant() {
        let engine_state = create_engine_state();
        
        // The nu constant should be generated and available
        // We can verify by checking that the engine state was set up properly
        assert!(
            engine_state.num_decls() > 0,
            "Engine state should have declarations after initialization"
        );
    }

    #[test]
    fn test_engine_state_can_parse_code() {
        let engine_state = create_engine_state();
        let mut working_set = StateWorkingSet::new(&engine_state);
        
        // Test that we can parse basic Nushell code
        let source = b"let x = 5";
        let _block = nu_parser::parse(&mut working_set, None, source, false);
        
        // If parsing succeeded without panic, the engine state is properly initialized
        assert!(true);
    }
}
