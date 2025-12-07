use nu_protocol::ast::{Expr, Expression, ExternalArgument};

use crate::{Fix, Violation, context::LintContext};

/// Return an iterator of borrowed slices for each external arg's inner text.
/// This avoids any allocation and does not prepend the spread prefix.
pub fn external_args_slices<'a>(
    args: &'a [ExternalArgument],
    context: &'a LintContext,
) -> impl Iterator<Item = &'a str> + 'a {
    args.iter().map(move |arg| match arg {
        ExternalArgument::Regular(expr) | ExternalArgument::Spread(expr) => {
            &context.source[expr.span.start..expr.span.end]
        }
    })
}

/// Type alias for a function that builds a fix for a specific external command
pub type FixBuilder = fn(
    cmd_text: &str,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix;

/// Detect a specific external command and suggest a builtin alternative
#[must_use]
pub fn detect_external_commands(
    context: &LintContext,
    external_cmd: &'static str,
    note: &'static str,
    fix_builder: Option<FixBuilder>,
) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| {
        if let Expr::ExternalCall(head, args) = &expr.expr {
            let cmd_text = &ctx.source[head.span.start..head.span.end];

            if cmd_text == external_cmd {
                let violation = create_violation(fix_builder, expr, ctx, cmd_text, note, args);

                return vec![violation];
            }
        }
        vec![]
    })
}

fn create_violation(
    fix_builder: Option<FixBuilder>,
    expr: &Expression,
    ctx: &LintContext<'_>,
    cmd_text: &str,
    note: &'static str,
    args: &[ExternalArgument],
) -> Violation {
    let message = format!("Consider using Nushell's built-ins instead of external '^{cmd_text}'");

    let suggestion = format!(
        "Replace external '^{cmd_text}' with equivalent Nushell pipeline.\nBuilt-in commands are \
         more portable, faster, and provide better error handling.\n\nNote: {note}"
    );

    let fix = fix_builder.map(|builder| builder(cmd_text, args, expr.span, ctx));

    let violation = Violation::new(message, expr.span)
        .with_primary_label(format!("external '{cmd_text}'"))
        .with_help(suggestion);

    match fix {
        Some(f) => violation.with_fix(f),
        None => violation,
    }
}
