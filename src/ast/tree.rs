use std::io::{self, Write};

/// Print AST with Debug formatting to stdout
pub fn print_ast(source: &str) -> io::Result<()> {
    use crate::engine::{LintEngine, parse_source};

    let engine_state = LintEngine::new_state();
    let (block, working_set, _offset) = parse_source(engine_state, source.as_bytes(), None);

    if !working_set.parse_errors.is_empty() {
        let mut stderr = io::stderr().lock();
        writeln!(stderr, "=== Parse Errors ===")?;
        for error in &working_set.parse_errors {
            writeln!(stderr, "{error:?}")?;
        }
        writeln!(stderr)?;
    }

    writeln!(io::stdout(), "{block:#?}")
}
