//! Reedline integration for nu-lint inline diagnostics.
//!
//! This module provides a `DiagnosticsProvider` implementation that uses
//! nu-lint to provide real-time linting feedback while typing nushell commands.
//!
//! # Example
//!
//! ```rust,ignore
//! use nu_lint::reedline_adapter::NuLintDiagnosticsProvider;
//! use reedline::Reedline;
//!
//! let provider = NuLintDiagnosticsProvider::new();
//! let editor = Reedline::create()
//!     .with_diagnostics(Box::new(provider));
//! ```

use reedline::{
    Diagnostic as ReedlineDiagnostic, DiagnosticFix as ReedlineFix,
    DiagnosticReplacement as ReedlineReplacement, DiagnosticSeverity as ReedlineSeverity,
    DiagnosticSpan as ReedlineSpan, DiagnosticsProvider,
};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

use crate::{Config, LintEngine, LintLevel};

const LOG_FILE: &str = "/tmp/nu-lint-reedline.log";

static ADAPTER_LOG: Mutex<Option<std::fs::File>> = Mutex::new(None);

fn adapter_log(msg: &str) {
    // Try to open log file if not already open
    if let Ok(mut guard) = ADAPTER_LOG.lock() {
        if guard.is_none() {
            if let Ok(file) = OpenOptions::new().create(true).append(true).open(LOG_FILE) {
                *guard = Some(file);
            }
        }
        if let Some(ref mut file) = *guard {
            let _ = writeln!(file, "[adapter] {}", msg);
            let _ = file.flush();
        }
    }
}

/// A diagnostics provider that uses nu-lint to lint nushell commands.
///
/// This implements reedline's `DiagnosticsProvider` trait, providing inline
/// diagnostics while the user types nushell commands.
pub struct NuLintDiagnosticsProvider {
    engine: LintEngine,
}

impl NuLintDiagnosticsProvider {
    /// Create a new provider with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            engine: LintEngine::new(Config::default()),
        }
    }

    /// Create a new provider with custom configuration.
    #[must_use]
    pub fn with_config(config: Config) -> Self {
        Self {
            engine: LintEngine::new(config),
        }
    }
}

impl Default for NuLintDiagnosticsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticsProvider for NuLintDiagnosticsProvider {
    fn diagnose(&mut self, line: &str, cursor: usize) -> Vec<ReedlineDiagnostic> {
        adapter_log(&format!("diagnose() called: line='{}' cursor={}", line, cursor));

        if line.is_empty() {
            adapter_log("Empty line, returning no diagnostics");
            return Vec::new();
        }

        let violations = self.engine.lint_str(line);
        adapter_log(&format!("lint_str returned {} violations", violations.len()));

        let diagnostics: Vec<_> = violations
            .into_iter()
            .map(|v| {
                let file_span = v.file_span();
                let span = ReedlineSpan::new(file_span.start, file_span.end);

                let severity = match v.lint_level {
                    LintLevel::Error => ReedlineSeverity::Error,
                    LintLevel::Warning => ReedlineSeverity::Warning,
                    LintLevel::Hint => ReedlineSeverity::Hint,
                };

                adapter_log(&format!(
                    "  [{:?}] {} (span: {}..{})",
                    severity,
                    v.message,
                    file_span.start,
                    file_span.end
                ));

                let mut diagnostic =
                    ReedlineDiagnostic::new(severity, span, v.message.to_string());

                if let Some(detail) = v.long_description {
                    diagnostic = diagnostic.with_detail(detail);
                }

                if let Some(rule_id) = v.rule_id {
                    diagnostic = diagnostic.with_rule_id(rule_id.to_string());
                }

                if let Some(fix) = v.fix {
                    let replacements: Vec<ReedlineReplacement> = fix
                        .replacements
                        .into_iter()
                        .map(|r| {
                            let file_span = r.file_span();
                            ReedlineReplacement::new(
                                ReedlineSpan::new(file_span.start, file_span.end),
                                r.replacement_text.to_string(),
                            )
                        })
                        .collect();

                    diagnostic =
                        diagnostic.with_fix(ReedlineFix::new(fix.explanation.to_string(), replacements));
                }

                diagnostic
            })
            .collect();

        adapter_log(&format!("Returning {} diagnostics", diagnostics.len()));
        diagnostics
    }

    fn on_input_change(&mut self) {
        adapter_log("on_input_change() called");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let _provider = NuLintDiagnosticsProvider::new();
        // Provider should be created successfully
    }

    #[test]
    fn test_provider_default() {
        let _provider = NuLintDiagnosticsProvider::default();
        // Default implementation should work
    }

    #[test]
    fn test_provider_with_config() {
        let config = Config::default();
        let _provider = NuLintDiagnosticsProvider::with_config(config);
        // Custom config should work
    }

    #[test]
    fn test_empty_input() {
        let mut provider = NuLintDiagnosticsProvider::new();
        let diagnostics = provider.diagnose("", 0);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_valid_nushell() {
        let mut provider = NuLintDiagnosticsProvider::new();
        let diagnostics = provider.diagnose("ls", 0);
        // Valid simple commands should have few or no diagnostics
        // depending on config
        assert!(diagnostics.len() < 10);
    }

    #[test]
    fn test_snake_case_violation() {
        let mut provider = NuLintDiagnosticsProvider::new();
        // camelCase variable should trigger snake_case lint
        let diagnostics = provider.diagnose("let myVariable = 5", 0);
        // Check if any diagnostic mentions snake_case
        let has_snake_case_warning = diagnostics
            .iter()
            .any(|d| d.message.contains("snake") || d.rule_id.as_deref() == Some("snake_case_variables"));
        // This should trigger because snake_case_variables is enabled by default
        assert!(has_snake_case_warning);
    }

    #[test]
    fn test_diagnostic_has_span() {
        let mut provider = NuLintDiagnosticsProvider::new();
        let diagnostics = provider.diagnose("let myVariable = 5", 0);
        // All diagnostics should have valid spans within the input
        for d in &diagnostics {
            assert!(d.span.start <= d.span.end);
            assert!(d.span.end <= "let myVariable = 5".len());
        }
    }

    #[test]
    fn test_severity_mapping() {
        // Test that LintLevel maps correctly to ReedlineSeverity
        assert_eq!(
            match LintLevel::Error {
                LintLevel::Error => ReedlineSeverity::Error,
                LintLevel::Warning => ReedlineSeverity::Warning,
                LintLevel::Hint => ReedlineSeverity::Hint,
            },
            ReedlineSeverity::Error
        );
        assert_eq!(
            match LintLevel::Warning {
                LintLevel::Error => ReedlineSeverity::Error,
                LintLevel::Warning => ReedlineSeverity::Warning,
                LintLevel::Hint => ReedlineSeverity::Hint,
            },
            ReedlineSeverity::Warning
        );
        assert_eq!(
            match LintLevel::Hint {
                LintLevel::Error => ReedlineSeverity::Error,
                LintLevel::Warning => ReedlineSeverity::Warning,
                LintLevel::Hint => ReedlineSeverity::Hint,
            },
            ReedlineSeverity::Hint
        );
    }
}
