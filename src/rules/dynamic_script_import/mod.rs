use nu_protocol::ast::{Argument, Expr, Expression, FullCellPath};

use crate::{
    ast::call::CallExt,
    config::LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct DynamicScriptImport;

impl DetectFix for DynamicScriptImport {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "dynamic_script_import"
    }

    fn explanation(&self) -> &'static str {
        "Dynamic import paths cannot be statically validated and may lead to runtime errors."
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/modules.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(context.detect(|expr, ctx| Self::check_expression(expr, ctx)))
    }
}

impl DynamicScriptImport {
    fn check_expression(expr: &Expression, ctx: &LintContext) -> Vec<Detection> {
        let Expr::Call(call) = &expr.expr else {
            return vec![];
        };

        let is_import_command = call.is_call_to_command("use", ctx)
            || call.is_call_to_command("source", ctx)
            || call.is_call_to_command("overlay use", ctx);

        if !is_import_command {
            return vec![];
        }
        let text = ctx.plain_text(call.span());
        log::debug!("Checking of `{text}` has a dynamic path");
        // Check if any positional argument is a dynamic expression (not a literal)
        let has_dynamic_path = call.arguments.iter().any(|arg| match arg {
            Argument::Positional(e) | Argument::Unknown(e) | Argument::Spread(e) => {
                let argument = ctx.plain_text(e.span);
                log::debug!("Checking whether argument `{argument}` is dynamic.");
                is_dynamic_expression(e)
            }
            Argument::Named(e) => e.2.as_ref().is_some_and(|e| {
                log::debug!("Checking whether named argument is dynamic.");
                is_dynamic_expression(e)
            }),
        });

        if !has_dynamic_path {
            return vec![];
        }

        let command_name = call.get_call_name(ctx);
        vec![
            Detection::from_global_span(
                format!(
                    "`{command_name}` uses a dynamic path that cannot be validated at lint time"
                ),
                expr.span,
            )
            .with_primary_label("dynamic import path"),
        ]
    }
}
fn is_dynamic_expression(expr: &Expression) -> bool {
    log::debug!(
        "Checking whether expression of type `{:#?}` is dynamic.",
        expr.expr
    );
    match &expr.expr {
        // Literal strings and filepaths are static
        // Variables, subexpressions, and pipelines are dynamic
        // String interpolation is dynamic if it contains expressions
        Expr::StringInterpolation(parts) => {
            log::debug!("Encountered string interpolation.");
            parts.iter().any(is_dynamic_expression)
        }
        Expr::List(list) => list.iter().any(|e| {
            log::debug!(
                "Encountered a list with element of the shape: {:#?}",
                e.expr()
            );

            is_dynamic_expression(e.expr())
        }),
        Expr::FullCellPath(e) => match &**e {
            FullCellPath {
                head:
                    Expression {
                        expr: Expr::List(list),
                        ..
                    },
                ..
            } => list.iter().any(|e| is_dynamic_expression(e.expr())),
            _ => true,
        },
        // Other literal types are static
        Expr::String(_)
        | Expr::RawString(_)
        | Expr::GlobPattern(_, _)
        | Expr::Filepath(_, _)
        | Expr::Int(_)
        | Expr::Float(_)
        | Expr::Bool(_)
        | Expr::Nothing => false,
        // Everything else - check conservatively
        _ => true,
    }
}
pub static RULE: &dyn Rule = &DynamicScriptImport;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
