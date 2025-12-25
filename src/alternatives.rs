use nu_protocol::{
    Span,
    ast::{Expr, Expression, ExternalArgument},
};

use crate::{context::LintContext, violation::Detection};

/// Return an iterator of borrowed slices for each external arg's inner text.
/// This avoids any allocation and does not prepend the spread prefix.
pub fn external_args_slices<'a>(
    args: &'a [ExternalArgument],
    context: &'a LintContext,
) -> impl Iterator<Item = &'a str> + 'a {
    args.iter().map(move |arg| match arg {
        ExternalArgument::Regular(expr) | ExternalArgument::Spread(expr) => {
            context.get_span_text(expr.span)
        }
    })
}

/// Fix data for external command alternatives
pub struct ExternalCmdFixData {
    pub args: Box<[ExternalArgument]>,
    pub expr_span: Span,
}

/// Detect a specific external command and suggest a builtin alternative.
/// Returns detected violations with fix data that can be used to generate
/// fixes.
#[must_use]
pub fn detect_external_commands(
    context: &LintContext,
    external_cmd: &'static str,
    note: &'static str,
) -> Vec<(Detection, ExternalCmdFixData)> {
    context.detect_with_fix_data(|expr, ctx| {
        if let Expr::ExternalCall(head, args) = &expr.expr {
            let cmd_text = ctx.get_span_text(head.span);

            if cmd_text == external_cmd {
                let detected = create_detected_violation(expr, cmd_text, note);
                let fix_data = ExternalCmdFixData {
                    args: args.clone(),
                    expr_span: expr.span,
                };

                return vec![(detected, fix_data)];
            }
        }
        vec![]
    })
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
