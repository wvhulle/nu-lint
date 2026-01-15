use nu_protocol::ast::{Expr, Expression, FindMapResult, Traverse};

use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    effect::{
        CommonEffect,
        builtin::{BuiltinEffect, has_builtin_side_effect},
    },
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn is_inside_try_block(context: &LintContext, span: nu_protocol::Span) -> bool {
    context
        .ast
        .find_map(context.working_set, &|expr| {
            let Expr::Call(call) = &expr.expr else {
                return FindMapResult::Continue;
            };
            if call.is_call_to_command("try", context)
                && expr.span.start <= span.start
                && expr.span.end >= span.end
            {
                return FindMapResult::Found(());
            }
            FindMapResult::Continue
        })
        .is_some()
}

fn check_call(expr: &Expression, context: &LintContext) -> Option<Detection> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    let name = call.get_call_name(context);
    let can_error = has_builtin_side_effect(
        &name,
        BuiltinEffect::CommonEffect(CommonEffect::LikelyErrors),
        context,
        call,
    );

    if !can_error || is_inside_try_block(context, expr.span) {
        return None;
    }

    Some(
        Detection::from_global_span(
            format!("'{name}' may fail. Wrap in 'try' block."),
            call.head,
        )
        .with_primary_label("command can error at runtime"),
    )
}

struct CatchBuiltinErrorTry;

impl DetectFix for CatchBuiltinErrorTry {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "catch_builtin_error_try"
    }

    fn short_description(&self) -> &'static str {
        "Catch runtime errors from built-in commands using 'try' blocks"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Commands like 'http get', 'open', 'from json', and file operations can fail at \
             runtime due to network issues, missing files, or invalid data. These are runtime \
             errors that can be caught with 'try' blocks. Without error handling, failures \
             produce cryptic downstream errors. Note: parse-time errors (like 'source' with a \
             missing file) cannot be caught by 'try', and 'exit' bypasses error handling entirely.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/try.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(context.detect(|expr, ctx| check_call(expr, ctx).into_iter().collect()))
    }
}

pub static RULE: &dyn Rule = &CatchBuiltinErrorTry;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
