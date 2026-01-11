//! Interactive nu-lint diagnostics demo using reedline.
//!
//! This example demonstrates real-time nushell linting while typing.
//! Try typing nushell commands to see inline warnings and errors.
//!
//! Run with: cargo run --example reedline_diagnostics --features reedline
//!
//! Debug output is written to /tmp/nu-lint-reedline.log

use nu_ansi_term::{Color, Style};
use nu_lint::reedline_adapter::NuLintDiagnosticsProvider;
use reedline::{
    DefaultPrompt, DiagnosticSeverity, DiagnosticsConfig, DiagnosticsDisplayMode, Reedline, Signal,
};
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::sync::Mutex;

const LOG_FILE: &str = "/tmp/nu-lint-reedline.log";

static LOG: Mutex<Option<std::fs::File>> = Mutex::new(None);

fn log(msg: &str) {
    if let Ok(mut guard) = LOG.lock() {
        if let Some(ref mut file) = *guard {
            let _ = writeln!(file, "{}", msg);
            let _ = file.flush();
        }
    }
}

fn main() -> io::Result<()> {
    // Initialize log file
    {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(LOG_FILE)?;
        *LOG.lock().unwrap() = Some(file);
    }

    log("=== Nu-lint Reedline Debug Session Started ===");

    // Configure diagnostics display
    // Note: debounce_ms(0) disables debouncing - diagnostics run on every keystroke
    // This is needed because reedline doesn't have periodic repaints
    let diagnostics_config = DiagnosticsConfig::new()
        .with_min_severity(DiagnosticSeverity::Hint) // Show all severities
        .with_display_mode(DiagnosticsDisplayMode::Both) // Inline + below
        .with_debounce_ms(0) // Disable debouncing (run on every keystroke)
        .with_max_below_lines(10) // Show up to 10 diagnostics below
        .with_error_style(Style::new().fg(Color::Red).underline())
        .with_warning_style(Style::new().fg(Color::Yellow).underline())
        .with_info_style(Style::new().fg(Color::Blue).underline())
        .with_hint_style(Style::new().fg(Color::Cyan).underline());

    log("Config created");

    // Create the nu-lint diagnostics provider
    let provider = NuLintDiagnosticsProvider::new();
    log("Provider created");

    // Create reedline with nu-lint diagnostics
    let mut line_editor = Reedline::create()
        .with_diagnostics(Box::new(provider))
        .with_diagnostics_config(diagnostics_config);

    log("Reedline created with diagnostics");

    let prompt = DefaultPrompt::default();

    println!("Nu-lint Inline Diagnostics Demo");
    println!("================================");
    println!();
    println!("Debug log: {}", LOG_FILE);
    println!();
    println!("Type nushell commands to see real-time linting feedback!");
    println!();
    println!("Try these examples:");
    println!("  - 'let myVariable = 5' -> snake_case warning");
    println!("  - 'let x = 1; $x' -> unused variable hint");
    println!("  - Invalid syntax will show parse errors");
    println!();
    println!("Press Ctrl+D or Ctrl+C to exit.");
    println!();

    log("Starting read_line loop");

    loop {
        log("Calling read_line...");
        match line_editor.read_line(&prompt)? {
            Signal::Success(buffer) => {
                log(&format!("read_line returned Success: '{}'", buffer));
                if buffer.trim().is_empty() {
                    continue;
                }
                println!("Entered: {buffer}");
                println!();
            }
            Signal::CtrlD | Signal::CtrlC => {
                log("read_line returned CtrlD/CtrlC");
                println!("\nGoodbye!");
                break Ok(());
            }
        }
    }
}
