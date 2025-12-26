use nu_protocol::{
    Span,
    ast::{Expr, Expression, ExternalArgument},
};

use crate::{context::LintContext, violation::Detection};

/// Fix data for external command alternatives
pub struct ExternalCmdFixData<'a> {
    pub arg_strings: Vec<&'a str>,
    pub expr_span: Span,
}

/// Detect a specific external command and suggest a builtin alternative.
/// Returns detected violations with fix data that can be used to generate
/// fixes.
#[must_use]
pub fn detect_external_commands<'context>(
    context: &'context LintContext,
    external_cmd: &'static str,
    note: &'static str,
) -> Vec<(Detection, ExternalCmdFixData<'context>)> {
    use nu_protocol::ast::Traverse;

    let mut results = Vec::new();

    context.ast.flat_map(
        context.working_set,
        &|expr| {
            if let Expr::ExternalCall(head, args) = &expr.expr {
                let cmd_text = context.get_span_text(head.span);

                if cmd_text == external_cmd {
                    let detected = create_detected_violation(expr, cmd_text, note);

                    let arg_strings: Vec<&str> = args
                        .iter()
                        .map(|arg| match arg {
                            ExternalArgument::Regular(expr) | ExternalArgument::Spread(expr) => {
                                context.get_span_text(expr.span)
                            }
                        })
                        .collect();

                    let fix_data = ExternalCmdFixData {
                        arg_strings,
                        expr_span: expr.span,
                    };

                    return vec![(detected, fix_data)];
                }
            }
            vec![]
        },
        &mut results,
    );

    results
}

fn create_detected_violation(expr: &Expression, cmd_text: &str, note: &'static str) -> Detection {
    let message = format!("Consider using Nushell's built-ins instead of external '^{cmd_text}'");

    let suggestion = format!(
        "Replace external '^{cmd_text}' with equivalent Nushell pipeline.\nBuilt-in commands are \
         more portable, faster, and provide better error handling.\n\nNote: {note}"
    );

    Detection::from_global_span(message, expr.span)
        .with_primary_label(format!("external '{cmd_text}'"))
        .with_help(suggestion)
}
