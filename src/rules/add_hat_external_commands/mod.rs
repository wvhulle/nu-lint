use nu_protocol::ast::{Expr, ExternalArgument};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    rules::remove_hat_not_builtin,
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

    fn short_description(&self) -> &'static str {
        "Always use the '^' prefix on external commands"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/running_externals.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Off
    }

    fn conflicts_with(&self) -> &'static [&'static dyn Rule] {
        static CONFLICTS: &[&dyn Rule] = &[remove_hat_not_builtin::RULE];
        CONFLICTS
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
                            ctx.span_text(head.span)
                        ),
                        expr.span,
                    )
                    .with_primary_label("missing '^' prefix"),
                    FixData {
                        cmd: ctx.span_text(head.span).into(),
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
                ExternalArgument::Regular(e) | ExternalArgument::Spread(e) => context.expr_text(e),
            })
            .collect::<Vec<_>>()
            .join(" ");

        let replacement = format!("^{} {args_text}", fix_data.cmd);

        Some(Fix {
            explanation: "Add missing '^' prefix".into(),
            replacements: vec![Replacement::new(fix_data.expr_span, replacement)],
        })
    }
}

pub static RULE: &dyn Rule = &AlwaysHatExtCall;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
