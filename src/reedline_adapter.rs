//! Reedline integration for nu-lint inline diagnostics.

use reedline::{
    Diagnostic as ReedlineDiagnostic, DiagnosticFix as ReedlineFix,
    DiagnosticReplacement as ReedlineReplacement, DiagnosticSeverity as ReedlineSeverity,
    DiagnosticSpan as ReedlineSpan, DiagnosticsProvider,
};

use crate::{Config, LintEngine, LintLevel};

/// A diagnostics provider that uses nu-lint to lint nushell commands.
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
    fn diagnose(&mut self, line: &str, _cursor: usize) -> Vec<ReedlineDiagnostic> {
        if line.is_empty() {
            return Vec::new();
        }

        self.engine
            .lint_str(line)
            .into_iter()
            .map(|v| {
                let file_span = v.file_span();
                let span = ReedlineSpan::new(file_span.start, file_span.end);

                let severity = match v.lint_level {
                    LintLevel::Error => ReedlineSeverity::Error,
                    LintLevel::Warning => ReedlineSeverity::Warning,
                    LintLevel::Hint => ReedlineSeverity::Hint,
                };

                let mut diagnostic = ReedlineDiagnostic::new(severity, span, v.message.to_string());

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

                    diagnostic = diagnostic
                        .with_fix(ReedlineFix::new(fix.explanation.to_string(), replacements));
                }

                diagnostic
            })
            .collect()
    }
}
