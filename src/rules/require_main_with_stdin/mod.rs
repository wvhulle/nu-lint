use crate::{
    LintLevel,
    ast::{expression::ExpressionExt, span::SpanExt},
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

    fn level(&self) -> LintLevel {
        LintLevel::Error
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

        let violations = context.detect(|expr, ctx| {
            // Skip if this expression is inside any function definition
            if expr
                .span
                .find_containing_function(&functions, ctx)
                .is_some()
            {
                return vec![];
            }

            // Check if this top-level expression uses $in (not $it in row conditions)
            if let Some(dollar_in_span) = expr.find_dollar_in_usage() {
                return vec![
                    Detection::from_global_span(
                        "Using $in outside of a main function",
                        dollar_in_span,
                    )
                    .with_primary_label("$in used here"),
                ];
            }
            vec![]
        });

        Self::no_fix(violations)
    }
}

pub static RULE: &dyn Rule = &RequireMainWithStdin;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
