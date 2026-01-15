use nu_protocol::ast::{Expr, Expression, ExternalArgument};

use crate::{
    config::LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct AvoidNuSubprocess;

impl DetectFix for AvoidNuSubprocess {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "avoid_nu_subprocess"
    }

    fn short_description(&self) -> &'static str {
        "Spawning `nu -c` from within a Nu script is redundant; call functions directly instead"
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Error)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(context.detect(|expr, ctx| Self::check_expression(expr, ctx)))
    }
}

impl AvoidNuSubprocess {
    fn check_expression(expr: &Expression, ctx: &LintContext) -> Vec<Detection> {
        let Expr::ExternalCall(head, args) = &expr.expr else {
            return vec![];
        };

        let cmd_name = ctx.plain_text(head.span);

        if !Self::is_nu_with_c_flag(cmd_name, args, ctx) {
            return vec![];
        }

        vec![
            Detection::from_global_span(
                "Avoid spawning `nu -c` subprocess from within a Nu script",
                expr.span,
            )
            .with_primary_label("subprocess spawned here"),
        ]
    }

    fn is_nu_with_c_flag(cmd_name: &str, args: &[ExternalArgument], ctx: &LintContext) -> bool {
        if cmd_name != "nu" {
            return false;
        }

        args.iter().any(|arg| {
            let ExternalArgument::Regular(expr) = arg else {
                return false;
            };
            let arg_text = ctx.plain_text(expr.span);
            arg_text == "-c" || arg_text == "--commands"
        })
    }
}

pub static RULE: &dyn Rule = &AvoidNuSubprocess;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
