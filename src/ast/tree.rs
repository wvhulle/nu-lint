/// Print AST with Debug formatting to stdout
pub fn print_ast(source: &str) {
    use crate::engine::{LintEngine, parse_source};

    let engine_state = LintEngine::new_state();
    let (block, working_set, _offset) = parse_source(engine_state, source.as_bytes(), None);

    if !working_set.parse_errors.is_empty() {
        eprintln!("=== Parse Errors ===");
        for error in &working_set.parse_errors {
            eprintln!("{error:?}");
        }
        eprintln!();
    }

    // Use Rust's built-in Debug trait for pretty-printing, just like Nu's ast
    // command
    println!("{block:#?}");
}
