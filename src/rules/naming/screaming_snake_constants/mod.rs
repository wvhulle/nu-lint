use nu_protocol::ast::Expr;

use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn is_valid_screaming_snake(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    first.is_ascii_uppercase()
        && chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

struct ScreamingSnakeConstants;

impl DetectFix for ScreamingSnakeConstants {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "screaming_snake_constants"
    }

    fn short_description(&self) -> &'static str {
        "Constants should use SCREAMING_SNAKE_CASE naming convention"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/style_guide.html#environment-variables")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let violations = context.detect(|expr, ctx| {
            let Expr::Call(call) = &expr.expr else {
                return vec![];
            };

            if !call.is_call_to_command("const", ctx) {
                return vec![];
            }

            let Some(var_arg) = call.get_first_positional_arg() else {
                return vec![];
            };

            let Expr::VarDecl(_) = &var_arg.expr else {
                return vec![];
            };

            let const_name = ctx.span_text(var_arg.span);

            if is_valid_screaming_snake(const_name) {
                vec![]
            } else {
                vec![
                    Detection::from_global_span(
                        format!(
                            "Constant '{const_name}' should use SCREAMING_SNAKE_CASE naming \
                             convention"
                        ),
                        var_arg.span,
                    )
                    .with_primary_label("non-SCREAMING_SNAKE_CASE"),
                ]
            }
        });
        Self::no_fix(violations)
    }
}

pub static RULE: &dyn Rule = &ScreamingSnakeConstants;
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
