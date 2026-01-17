use nu_protocol::ast::{Expr, Expression};

use crate::{
    ast::call::CallExt,
    config::LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct UseOverSource;

impl DetectFix for UseOverSource {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "source_to_use"
    }

    fn short_description(&self) -> &'static str {
        "`source` replaceable with `use`"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "The 'use' command imports exported module definitions with namespace control and \
             selective imports. The 'source' command executes scripts directly, causing namespace \
             pollution. Only use 'source' for scripts with side effects, not for importing \
             definitions.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/modules/using_modules.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let detections = context.detect(|expr, ctx| {
            if Self::is_source_command(expr, ctx) {
                vec![Self::create_detection(expr)]
            } else {
                vec![]
            }
        });
        Self::no_fix(detections)
    }
}

impl UseOverSource {
    fn is_source_command(expr: &Expression, ctx: &LintContext) -> bool {
        let Expr::Call(call) = &expr.expr else {
            return false;
        };
        call.is_call_to_command("source", ctx)
    }

    fn create_detection(expr: &Expression) -> Detection {
        Detection::from_global_span(
            "Use 'use' instead of 'source' for importing modules",
            expr.span,
        )
        .with_primary_label("'source' command used here")
    }
}

pub static RULE: &dyn Rule = &UseOverSource;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
