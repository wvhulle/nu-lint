use nu_protocol::ast::{Argument, Expr, Expression};

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

        // Check if any positional argument is a dynamic expression (not a literal)
        let has_dynamic_path = call.arguments.iter().any(|arg| {
            let arg_expr = match arg {
                Argument::Positional(e) | Argument::Unknown(e) | Argument::Spread(e) => e,
                Argument::Named(_) => return false,
            };

            Self::is_dynamic_expression(arg_expr)
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
            .with_primary_label("dynamic import path")
            .with_help(
                "This import uses a computed path (variable, subexpression, or pipeline). nu-lint \
                 cannot statically analyze the imported file, which may cause:\n- Parse errors \
                 from the imported file to appear as false positives\n- Missing validation of the \
                 imported module's structure",
            ),
        ]
    }

    fn is_dynamic_expression(expr: &Expression) -> bool {
        match &expr.expr {
            // Literal strings and filepaths are static
            // Variables, subexpressions, and pipelines are dynamic
            // String interpolation is dynamic if it contains expressions
            Expr::StringInterpolation(parts) => parts
                .iter()
                .any(|part| !matches!(part.expr, Expr::String(_))),
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
}

pub static RULE: &dyn Rule = &DynamicScriptImport;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
