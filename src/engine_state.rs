/// Shared engine state initialization code.
///
/// This module provides reusable engine state initialization that can be used
/// by both the linting engine and the LSP server, similar to how nu-lsp in
/// the main nushell binary initializes its engine state.
use std::env;

use nu_protocol::{
    Span, Value,
    engine::{EngineState, StateWorkingSet},
};

/// Initialize a Nushell engine state with all standard commands and context.
///
/// This creates a minimal but complete engine state suitable for parsing and
/// evaluating Nushell code. The initialization follows the same pattern as
/// nu-lsp in the main nushell binary.
///
/// The engine state includes:
/// - Default language context (core commands)
/// - Shell command context (filesystem, process commands, etc.)
/// - CLI context (REPL commands)
/// - PWD environment variable
/// - Print command
/// - $nu constant
///
/// Optionally, the standard library can be loaded if needed for more complete
/// command availability, though this adds initialization time.
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
    // This is what nu-lsp does in the main nushell binary
    if let Err(e) = nu_std::load_standard_library(&mut engine_state) {
        log::warn!("Failed to load standard library: {e}");
    }

    engine_state
}
