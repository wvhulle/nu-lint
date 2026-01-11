//! Debug diagnostics to understand what's happening
//!
//! Run with: cargo run --example debug_diagnostics --features reedline

use nu_lint::reedline_adapter::NuLintDiagnosticsProvider;
use reedline::DiagnosticsProvider;

fn main() {
    println!("Testing NuLintDiagnosticsProvider directly...\n");

    let mut provider = NuLintDiagnosticsProvider::new();

    let test_inputs = [
        "ls",
        "let myVariable = 5",
        "let x = 1",
        "def foo [] { }",
        "let camelCase = 10",
    ];

    for input in &test_inputs {
        println!("Input: '{}'", input);
        let diagnostics = provider.diagnose(input, 0);
        if diagnostics.is_empty() {
            println!("  No diagnostics\n");
        } else {
            for d in &diagnostics {
                println!(
                    "  [{:?}] {} (span: {}..{})",
                    d.severity, d.message, d.span.start, d.span.end
                );
                if let Some(rule) = &d.rule_id {
                    println!("    rule: {}", rule);
                }
            }
            println!();
        }
    }
}
