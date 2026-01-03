use nu_protocol::ast::{Expr, ExternalArgument};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

/// Data needed to generate a fix for unnecessary hat violations
pub struct FixData {
    cmd: Box<str>,
    args: Box<[ExternalArgument]>,
    expr_span: nu_protocol::Span,
}

struct AlwaysHatExtCall;

impl DetectFix for AlwaysHatExtCall {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "add_hat_external_commands"
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

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| {
            let Expr::ExternalCall(head, args) = &expr.expr else {
                return vec![];
            };

            let has_hat_prefix = expr.span.start + 1 == head.span.start;
            if !has_hat_prefix {
                return vec![(
                    Detection::from_global_span(
                        format!(
                            "External command '{}' is missing '^' prefix",
                            ctx.plain_text(head.span)
                        ),
                        expr.span,
                    )
                    .with_primary_label("missing '^' prefix")
                    .with_help("Use '^' prefix to indicate external command"),
                    FixData {
                        cmd: ctx.plain_text(head.span).into(),
                        args: args.to_vec().into_boxed_slice(),
                        expr_span: expr.span,
                    },
                )];
            }
            vec![]
        })
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let args_text: String = fix_data
            .args
            .iter()
            .map(|arg| match arg {
                ExternalArgument::Regular(e) | ExternalArgument::Spread(e) => {
                    context.plain_text(e.span)
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        let replacement = format!("^{} {args_text}", fix_data.cmd);

        Some(Fix::with_explanation(
            "Add missing '^' prefix",
            vec![Replacement::new(fix_data.expr_span, replacement)],
        ))
    }
}

pub static RULE: &dyn Rule = &AlwaysHatExtCall;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
