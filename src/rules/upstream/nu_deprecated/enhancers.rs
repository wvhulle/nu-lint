//! Enhancers for deprecated nushell features.
//!
//! Each enhancer matches specific deprecation patterns and provides fixes
//! and/or additional context.

use nu_protocol::{ParseWarning, Span};

use crate::{
    context::LintContext,
    violation::{Fix, Replacement},
};

/// Enhancement that can be applied to an upstream detection.
#[derive(Default)]
pub struct Enhancement {
    /// Additional notes to append to the message
    pub notes: Vec<String>,
    /// Extra labeled spans to add
    pub extra_labels: Vec<(Span, String)>,
    /// Optional auto-fix
    pub fix: Option<Fix>,
}

/// Try to enhance a deprecation warning with fixes and notes.
pub fn enhance(warning: &ParseWarning, context: &LintContext) -> Option<Enhancement> {
    let ParseWarning::Deprecated { label, span, .. } = warning;

    // get --ignore-errors / -i deprecation (renamed to --optional / -o in 0.106.0)
    if label.contains("get --ignore-errors") {
        let source = context.span_text(*span);
        let replacement = source
            .replace("--ignore-errors", "--optional")
            .replace("-i", "-o");

        return Some(Enhancement {
            notes: vec!["The --optional (-o) flag requires nushell >= 0.106.0".into()],
            extra_labels: vec![],
            fix: Some(Fix {
                explanation: "Replace with --optional (-o)".into(),
                replacements: vec![Replacement::new(*span, replacement)],
            }),
        });
    }

    None
}
