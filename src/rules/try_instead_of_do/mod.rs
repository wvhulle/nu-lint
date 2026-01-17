use nu_protocol::{
    Span,
    ast::{Expr, Expression, FindMapResult, Traverse},
};

use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    effect::{
        CommonEffect,
        builtin::can_error,
        external::{ExternEffect, has_external_side_effect},
    },
    rule::{DetectFix, Rule},
    violation::Detection,
};

enum ErrorSource {
    External(Span),
    Builtin(Span, String),
}

fn find_error_prone_command(expr: &Expression, context: &LintContext) -> Option<ErrorSource> {
    expr.find_map(context.working_set, &|inner_expr| match &inner_expr.expr {
        Expr::ExternalCall(head, args) => {
            let cmd_name = context.expr_text(head);
            if has_external_side_effect(
                cmd_name,
                ExternEffect::CommonEffect(CommonEffect::FailsInNormalCircumstances),
                context,
                args,
            ) {
                return FindMapResult::Found(ErrorSource::External(head.span));
            }
            FindMapResult::Found(ErrorSource::External(head.span))
        }
        Expr::Call(call) => {
            let cmd_name = call.get_call_name(context);
            if can_error(&cmd_name, context, call) {
                log::debug!("Found error-prone builtin: {cmd_name}");
                return FindMapResult::Found(ErrorSource::Builtin(call.head, cmd_name));
            }
            FindMapResult::Continue
        }
        _ => FindMapResult::Continue,
    })
}

struct TryInsteadOfDo;

impl DetectFix for TryInsteadOfDo {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "try_instead_of_do"
    }

    fn short_description(&self) -> &'static str {
        "Use 'try' blocks instead of 'do' blocks for error-prone operations"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/try.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let violations = context.detect(|expr, ctx| {
            let Expr::Call(call) = &expr.expr else {
                return vec![];
            };

            if !call.is_call_to_command("do", ctx) {
                return vec![];
            }

            let Some(block_arg) = call.get_positional_arg(0) else {
                return vec![];
            };

            let Some(error_source) = find_error_prone_command(block_arg, ctx) else {
                return vec![];
            };

            let do_span = Span::new(expr.span.start, expr.span.start + 2);
            let (error_span, error_label) = match &error_source {
                ErrorSource::External(span) => (*span, "external command can fail".to_string()),
                ErrorSource::Builtin(span, name) => (*span, format!("`{name}` can error")),
            };

            vec![
                Detection::from_global_span(
                    "Use 'try' instead of 'do' for error-prone operations",
                    do_span,
                )
                .with_primary_label("do keyword")
                .with_extra_label(error_label, error_span),
            ]
        });
        Self::no_fix(violations)
    }
}

pub static RULE: &dyn Rule = &TryInsteadOfDo;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
