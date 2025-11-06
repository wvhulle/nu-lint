use core::hash::BuildHasher;
use std::collections::HashMap;

use nu_protocol::ast::{Expr, Expression, ExternalArgument};

use crate::{
    context::LintContext,
    violation::{Fix, RuleViolation},
};

/// Extract external command arguments as strings
#[must_use]
pub fn extract_external_args(args: &[ExternalArgument], context: &LintContext) -> Vec<String> {
    args.iter()
        .map(|arg| match arg {
            ExternalArgument::Regular(expr) => {
                context.source[expr.span.start..expr.span.end].to_string()
            }
            ExternalArgument::Spread(expr) => {
                format!("...{}", &context.source[expr.span.start..expr.span.end])
            }
        })
        .collect()
}

/// Metadata about a builtin alternative to an external command
pub struct BuiltinAlternative {
    pub command: &'static str,
    pub note: Option<&'static str>,
}

impl BuiltinAlternative {
    #[must_use]
    pub const fn simple(command: &'static str) -> Self {
        Self {
            command,
            note: None,
        }
    }

    #[must_use]
    pub const fn with_note(command: &'static str, note: &'static str) -> Self {
        Self {
            command,
            note: Some(note),
        }
    }
}

/// Type alias for a function that builds a fix for a specific external command
pub type FixBuilder = fn(
    cmd_text: &str,
    alternative: &BuiltinAlternative,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix;

/// Detect external commands with builtin alternatives
#[must_use]
pub fn detect_external_commands<S: BuildHasher>(
    context: &LintContext,
    rule_id: &'static str,
    alternatives: &HashMap<&'static str, BuiltinAlternative, S>,
    fix_builder: Option<FixBuilder>,
) -> Vec<RuleViolation> {
    context.collect_rule_violations(|expr, ctx| {
        if let Expr::ExternalCall(head, args) = &expr.expr {
            let cmd_text = &ctx.source[head.span.start..head.span.end];

            if let Some(alternative) = alternatives.get(cmd_text) {
                let violation =
                    create_violation(rule_id, fix_builder, expr, ctx, cmd_text, alternative, args);

                return vec![violation];
            }
        }
        vec![]
    })
}

fn create_violation(
    rule_id: &'static str,
    fix_builder: Option<FixBuilder>,
    expr: &Expression,
    ctx: &LintContext<'_>,
    cmd_text: &str,
    alternative: &BuiltinAlternative,
    args: &[ExternalArgument],
) -> RuleViolation {
    let message = format!(
        "Consider using Nushell's built-in '{}' instead of external '^{}'",
        alternative.command, cmd_text
    );

    let suggestion = alternative.note.map_or_else(
        || {
            format!(
                "Replace '^{}' with built-in command: {}\nBuilt-in commands are more portable, \
                 faster, and provide better error handling.",
                cmd_text, alternative.command
            )
        },
        |note| {
            format!(
                "Replace '^{}' with built-in command: {}\nBuilt-in commands are more portable, \
                 faster, and provide better error handling.\n\nNote: {note}",
                cmd_text, alternative.command
            )
        },
    );

    let fix = fix_builder.map(|builder| builder(cmd_text, alternative, args, expr.span, ctx));

    let violation =
        RuleViolation::new_dynamic(rule_id, message, expr.span).with_suggestion_dynamic(suggestion);

    match fix {
        Some(f) => violation.with_fix(f),
        None => violation,
    }
}
