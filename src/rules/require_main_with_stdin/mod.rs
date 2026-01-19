use std::ops::ControlFlow;

use nu_protocol::ast::Expr;

use crate::{
    LintLevel,
    ast::{expression::is_dollar_in_var, span::SpanExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct RequireMainWithStdin;

impl DetectFix for RequireMainWithStdin {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "require_main_with_stdin"
    }

    fn short_description(&self) -> &'static str {
        "Scripts using $in must define a main function"
    }

    fn source_link(&self) -> Option<&'static str> {
        None
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Error)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let functions = context.custom_commands();
        let has_main = functions
            .iter()
            .any(super::super::ast::declaration::CustomCommandDef::is_main);

        // Only check top-level code if there's no main function
        if has_main {
            return Self::no_fix(vec![]);
        }

        let mut violations = Vec::new();

        context.traverse_with_parent(|expr, _parent| {
            // Skip closures - don't recurse into them (closures have their own $in scope)
            if matches!(expr.expr, Expr::Closure(_)) {
                return ControlFlow::Break(());
            }

            // Skip function definitions - don't recurse into them
            if expr
                .span
                .find_containing_function(&functions, context)
                .is_some()
            {
                return ControlFlow::Break(());
            }

            // Check for $in variable usage
            // Note: Nu represents $in usage as either Expr::Var or Expr::Collect
            let is_dollar_in = match &expr.expr {
                Expr::Var(var_id) => is_dollar_in_var(*var_id),
                Expr::Collect(_, _) => true, // Collect always represents $in pipeline input
                _ => false,
            };

            if is_dollar_in {
                violations.push(
                    Detection::from_global_span("Using $in outside of a main function", expr.span)
                        .with_primary_label("$in used here"),
                );
            }

            ControlFlow::Continue(())
        });

        Self::no_fix(violations)
    }
}

pub static RULE: &dyn Rule = &RequireMainWithStdin;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
