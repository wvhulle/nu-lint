use nu_protocol::ast::{Expr, Expression};

use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn check_call(expr: &Expression, context: &LintContext) -> Option<Detection> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    if call.get_call_name(context) != "default" {
        return None;
    }

    let arg = call.get_first_positional_arg()?;
    let is_empty_string = matches!(&arg.expr, Expr::String(s) | Expr::RawString(s) if s.is_empty());

    if !is_empty_string {
        return None;
    }

    Some(
        Detection::from_global_span(
            r#"'default ""' turns a missing value into an indistinguishable empty string"#,
            expr.span,
        )
        .with_primary_label("masks a missing value as an empty string"),
    )
}

struct DefaultEmptyStringMasksMissing;

impl DetectFix for DefaultEmptyStringMasksMissing {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "default_empty_string_masks_missing"
    }

    fn short_description(&self) -> &'static str {
        r#"Avoid '| default ""'; it masks a missing value as an empty string"#
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "'default' (without '--empty') only replaces 'null' values, so 'default \"\"' \
             silently turns a missing value into an indistinguishable empty string. Handle the \
             null case directly instead, e.g. 'match $x { null => ... _ => ... }' or 'if $x == \
             null { error make {...} }'.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/default.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(context.detect(|expr, ctx| check_call(expr, ctx).into_iter().collect()))
    }
}

pub static RULE: &dyn Rule = &DefaultEmptyStringMasksMissing;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
