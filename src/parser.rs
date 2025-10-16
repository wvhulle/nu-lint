use nu_parser::parse;
use nu_protocol::Span;
use nu_protocol::ast::Block;
use nu_protocol::engine::{EngineState, StateWorkingSet};

/// Parse Nushell source code into an AST and return both the Block and `StateWorkingSet`.
///
/// The `StateWorkingSet` contains the delta with newly defined declarations (functions, aliases, etc.)
/// which is essential for AST-based linting rules that need to inspect function signatures,
/// parameter types, and other semantic information.
///
/// AST-based rules can:
/// - Inspect function signatures for parameter ordering, types, and counts
/// - Check for documentation comments on declarations
/// - Analyze control flow and variable usage
/// - Detect semantic issues that regex cannot catch
///
/// # Performance Note
/// This function reuses the provided `EngineState` instead of creating a new one,
/// which significantly improves performance when linting multiple files.
pub fn parse_source<'a>(
    engine_state: &'a EngineState,
    source: &[u8],
) -> (Block, StateWorkingSet<'a>) {
    let mut working_set = StateWorkingSet::new(engine_state);
    let block = parse(&mut working_set, None, source, false);

    ((*block).clone(), working_set)
}

#[must_use]
pub fn span_to_range(source: &str, span: Span) -> (usize, usize) {
    let start = span.start.min(source.len());
    let end = span.end.min(source.len());
    (start, end)
}

#[must_use]
pub fn get_span_contents(source: &str, span: Span) -> &str {
    let (start, end) = span_to_range(source, span);
    &source[start..end]
}
