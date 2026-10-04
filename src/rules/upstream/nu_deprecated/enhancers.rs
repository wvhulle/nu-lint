//! Enhancers for deprecated nushell features.
//!
//! Each enhancer matches specific deprecation patterns and provides fixes
//! and/or additional context.

use nu_protocol::{
    ParseWarning, Span,
    ast::{Expr, Traverse},
};

use crate::{
    ast::call::CallExt,
    context::LintContext,
    violation::{Fix, Replacement},
};

struct RenamedCommand {
    deprecated: &'static str,
    successor: &'static str,
    since: &'static str,
}

const RENAMED_COMMANDS: &[RenamedCommand] = &[
    RenamedCommand {
        deprecated: "str upcase",
        successor: "str uppercase",
        since: "0.114.0",
    },
    RenamedCommand {
        deprecated: "str downcase",
        successor: "str lowercase",
        since: "0.114.0",
    },
];

struct RenamedCall {
    head: Span,
    renamed: &'static RenamedCommand,
}

fn renamed_call(warning_span: Span, context: &LintContext) -> Option<RenamedCall> {
    let mut calls = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| match &expr.expr {
            Expr::Call(call) if call.span() == warning_span => {
                let name = call.get_call_name(context);
                RENAMED_COMMANDS
                    .iter()
                    .find(|renamed| renamed.deprecated == name)
                    .map(|renamed| RenamedCall {
                        head: call.head,
                        renamed,
                    })
                    .into_iter()
                    .collect()
            }
            _ => vec![],
        },
        &mut calls,
    );
    calls.into_iter().next()
}

fn enhance_renamed_command(warning_span: Span, context: &LintContext) -> Option<Enhancement> {
    let RenamedCall { head, renamed } = renamed_call(warning_span, context)?;
    Some(Enhancement {
        notes: vec![format!(
            "`{}` requires nushell >= {}",
            renamed.successor, renamed.since
        )],
        extra_labels: vec![],
        fix: Some(Fix {
            explanation: format!("Replace with `{}`", renamed.successor).into(),
            replacements: vec![Replacement::new(head, renamed.successor)],
        }),
    })
}

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

    enhance_renamed_command(*span, context)
}
