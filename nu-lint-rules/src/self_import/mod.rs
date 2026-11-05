use nu_lint_analysis::{
    ast::call::CallExt,
    config::LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};
use nu_protocol::{
    ENV_VARIABLE_ID, NU_VARIABLE_ID, Span,
    ast::{Argument, Expr, Expression, PathMember, Traverse},
};

struct AvoidSelfImport;

impl DetectFix for AvoidSelfImport {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "self_import"
    }

    fn short_description(&self) -> &'static str {
        "Circular import: script imports itself"
    }

    fn level(&self) -> LintLevel {
        LintLevel::Error
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
            call.builtin_name(ctx) == Some("use") || call.builtin_name(ctx) == Some("source");
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

            Self::reads_current_file(arg_expr, ctx)
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

    fn is_current_file_reference(expr: &Expression, ctx: &LintContext) -> bool {
        match &expr.expr {
            Expr::FullCellPath(cell_path) => {
                let first_member = match cell_path.tail.first() {
                    Some(PathMember::String { val, .. }) => val.as_str(),
                    _ => return false,
                };
                match cell_path.head.expr {
                    Expr::Var(ENV_VARIABLE_ID) => first_member == "CURRENT_FILE",
                    Expr::Var(NU_VARIABLE_ID) => first_member == "current-file",
                    _ => false,
                }
            }
            Expr::Call(call) => call.builtin_name(ctx) == Some("path self"),
            _ => false,
        }
    }

    fn reads_current_file(arg_expr: &Expression, ctx: &LintContext) -> bool {
        let mut references: Vec<Span> = Vec::new();
        arg_expr.flat_map(
            ctx.working_set,
            &|expr| {
                if Self::is_current_file_reference(expr, ctx) {
                    vec![expr.span]
                } else {
                    vec![]
                }
            },
            &mut references,
        );
        !references.is_empty()
    }
}

pub static RULE: &dyn Rule = &AvoidSelfImport;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
