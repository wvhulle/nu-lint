use nu_protocol::ast::{Expr, ExternalArgument};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

/// Data needed to generate a fix for unnecessary hat violations
pub struct UnnecessaryHatFixData {
    cmd: Box<str>,
    args: Box<[ExternalArgument]>,
    expr_span: nu_protocol::Span,
}

fn has_builtin(name: &str, ctx: &LintContext) -> bool {
    ctx.engine_state.find_decl(name.as_bytes(), &[]).is_some()
}

struct UnnecessaryHat;

impl DetectFix for UnnecessaryHat {
    type FixInput = UnnecessaryHatFixData;

    fn id(&self) -> &'static str {
        "unnecessary_hat"
    }

    fn explanation(&self) -> &'static str {
        "Detect unnecessary '^' prefix on external commands"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/external_commands.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
        context.detect_with_fix_data(|expr, ctx| {
            let Expr::ExternalCall(head, args) = &expr.expr else {
                return vec![];
            };

            let has_hat_prefix = expr.span.start + 1 == head.span.start;
            if !has_hat_prefix {
                return vec![];
            }

            let cmd = ctx.get_span_text(head.span);
            if has_builtin(cmd, ctx) {
                return vec![];
            }

            let hat_span = nu_protocol::Span::new(expr.span.start, expr.span.start + 1);

            let violation = Detection::from_global_span(
                format!("Unnecessary '^' prefix on external command '{cmd}'"),
                hat_span,
            )
            .with_primary_label("redundant prefix")
            .with_extra_label("has no built-in equivalent", head.span)
            .with_help(format!(
                "The '^' prefix is only needed when a built-in command with the same name exists. \
                 '{cmd}' has no built-in equivalent, so the prefix is redundant."
            ));

            let fix_data = UnnecessaryHatFixData {
                cmd: cmd.into(),
                args: args.to_vec().into_boxed_slice(),
                expr_span: expr.span,
            };

            vec![(violation, fix_data)]
        })
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput) -> Option<Fix> {
        let args_text: String = fix_data
            .args
            .iter()
            .map(|arg| match arg {
                ExternalArgument::Regular(e) | ExternalArgument::Spread(e) => {
                    context.get_span_text(e.span)
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        let replacement = if args_text.is_empty() {
            fix_data.cmd.to_string()
        } else {
            format!("{} {args_text}", fix_data.cmd)
        };

        Some(Fix::with_explanation(
            "Remove unnecessary '^' prefix",
            vec![Replacement::new(fix_data.expr_span, replacement)],
        ))
    }
}

pub static RULE: &dyn Rule = &UnnecessaryHat;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
