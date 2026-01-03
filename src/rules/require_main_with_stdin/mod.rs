use nu_protocol::ast::{Argument, Expr, Expression};

use crate::{
    LintLevel,
    ast::{expression::is_pipeline_input_var, span::SpanExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

/// Check if an expression uses $in (pipeline input variable)
fn uses_pipeline_input(expr: &Expression, context: &LintContext) -> bool {
    match &expr.expr {
        Expr::Var(var_id) => {
            let var = context.working_set.get_variable(*var_id);
            is_pipeline_input_var(*var_id, context) && var.const_val.is_none()
        }
        Expr::BinaryOp(left, _, right) => {
            uses_pipeline_input(left, context) || uses_pipeline_input(right, context)
        }
        Expr::UnaryNot(inner) => uses_pipeline_input(inner, context),
        Expr::Collect(_var_id, _inner_expr) => true,
        Expr::Call(call) => call.arguments.iter().any(|arg| {
            if let Argument::Positional(arg_expr) | Argument::Named((_, _, Some(arg_expr))) = arg {
                uses_pipeline_input(arg_expr, context)
            } else {
                false
            }
        }),
        Expr::FullCellPath(cell_path) => uses_pipeline_input(&cell_path.head, context),
        Expr::Subexpression(block_id) => {
            let block = context.working_set.get_block(*block_id);
            block.pipelines.iter().any(|pipeline| {
                pipeline
                    .elements
                    .iter()
                    .any(|elem| uses_pipeline_input(&elem.expr, context))
            })
        }
        _ => false,
    }
}

struct RequireMainWithStdin;

impl DetectFix for RequireMainWithStdin {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "require_main_with_stdin"
    }

    fn explanation(&self) -> &'static str {
        "Scripts using $in must define a main function"
    }

    fn doc_url(&self) -> Option<&'static str> {
        None
    }

    fn level(&self) -> LintLevel {
        LintLevel::Error
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let functions = context.collect_function_definitions();
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

            // Check if this top-level expression uses $in
            if uses_pipeline_input(expr, ctx) {
                return vec![
                    Detection::from_global_span("Using $in outside of a main function", expr.span)
                        .with_primary_label("$in used here")
                        .with_help(
                            "When using $in to read from stdin/pipeline, you must define a 'def \
                             main [] { ... }' function. The $in variable is only available in \
                             pipeline contexts like function bodies.",
                        ),
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
