use nu_protocol::ast::{Argument, Expr, Expression};

use crate::{
    ast::call::CallExt,
    config::LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct AvoidSelfImport;

impl DetectFix for AvoidSelfImport {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "avoid_self_import"
    }

    fn short_description(&self) -> &'static str {
        "Avoid importing the current script from itself; functions are already available in scope"
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Error)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(context.detect(|expr, ctx| Self::check_expression(expr, ctx)))
    }
}

impl AvoidSelfImport {
    fn check_expression(expr: &Expression, ctx: &LintContext) -> Vec<Detection> {
        let Expr::Call(call) = &expr.expr else {
            return vec![];
        };

        let is_use_or_source =
            call.is_call_to_command("use", ctx) || call.is_call_to_command("source", ctx);
        if !is_use_or_source {
            return vec![];
        }

        let has_self_reference = call.arguments.iter().any(|arg| {
            let arg_expr = match arg {
                Argument::Positional(e)
                | Argument::Unknown(e)
                | Argument::Spread(e)
                | Argument::Named((_, _, Some(e))) => e,
                Argument::Named(_) => return false,
            };

            let arg_text = ctx.plain_text(arg_expr.span);
            Self::contains_current_file_reference(arg_text)
        });

        if !has_self_reference {
            return vec![];
        }

        let command_name = call.get_call_name(ctx);
        vec![
            Detection::from_global_span(
                format!(
                    "Avoid `{command_name}` with reference to current file (self-import pattern)"
                ),
                expr.span,
            )
            .with_primary_label("self-import here"),
        ]
    }

    fn contains_current_file_reference(text: &str) -> bool {
        text.contains("$env.CURRENT_FILE")
            || text.contains("$nu.current-file")
            || text.contains("(path self)")
    }
}

pub static RULE: &dyn Rule = &AvoidSelfImport;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
