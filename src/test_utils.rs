//! Shared test utilities for rule tests

/// Create an `EngineState` with the Nushell standard library for testing
/// This includes all builtin commands needed for AST-based rules
#[must_use]
pub fn create_engine_with_stdlib() -> nu_protocol::engine::EngineState {
    let engine_state = nu_cmd_lang::create_default_context();
    nu_command::add_shell_command_context(engine_state)
}
