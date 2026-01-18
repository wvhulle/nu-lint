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

struct RemoveHatNotBuiltin;

impl DetectFix for RemoveHatNotBuiltin {
    type FixInput<'a> = UnnecessaryHatFixData;

    fn id(&self) -> &'static str {
        "remove_hat_not_builtin"
    }

    fn short_description(&self) -> &'static str {
        "Detect unnecessary '^' prefix on external commands"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/running_externals.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| {
            let Expr::ExternalCall(head, args) = &expr.expr else {
                return vec![];
            };

            let has_hat_prefix = expr.span.start + 1 == head.span.start;
            if !has_hat_prefix {
                return vec![];
            }

            // If the command head is a variable or expression, the hat is necessary
            // because the runtime value might conflict with a builtin command
            if matches!(
                &head.expr,
                Expr::Var(_)
                    | Expr::FullCellPath(_)
                    | Expr::Subexpression(_)
                    | Expr::StringInterpolation(_)
            ) {
                return vec![];
            }

            let cmd = ctx.expr_text(head);
            if has_builtin(cmd, ctx) {
                return vec![];
            }

            let hat_span = nu_protocol::Span::new(expr.span.start, expr.span.start + 1);

            let violation = Detection::from_global_span(
                format!("Unnecessary '^' prefix on external command '{cmd}'"),
                hat_span,
            )
            .with_primary_label("redundant prefix")
            .with_extra_label("has no built-in equivalent", head.span);

            let fix_data = UnnecessaryHatFixData {
                cmd: cmd.into(),
                args: args.to_vec().into_boxed_slice(),
                expr_span: expr.span,
            };

            vec![(violation, fix_data)]
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

        let replacement = if args_text.is_empty() {
            fix_data.cmd.to_string()
        } else {
            format!("{} {args_text}", fix_data.cmd)
        };

        Some(Fix {
            explanation: "Remove unnecessary '^' prefix".into(),
            replacements: vec![Replacement::new(fix_data.expr_span, replacement)],
        })
    }
}

pub static RULE: &dyn Rule = &RemoveHatNotBuiltin;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
