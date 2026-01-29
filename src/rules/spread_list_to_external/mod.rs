use nu_protocol::{
    Type,
    ast::{Expr, ExternalArgument},
};

use crate::{
    LintLevel,
    ast::expression::ExpressionExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct SpreadListToExternal;

impl DetectFix for SpreadListToExternal {
    type FixInput<'a> = nu_protocol::Span;

    fn id(&self) -> &'static str {
        "spread_list_to_external"
    }

    fn short_description(&self) -> &'static str {
        "List variables passed to external commands should be spread with `...`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/operators.html#spread-operator")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| {
            let Expr::ExternalCall(_head, args) = &expr.expr else {
                return vec![];
            };

            let mut violations = Vec::new();

            for arg in args {
                // Only check Regular arguments, not already-Spread ones
                let ExternalArgument::Regular(arg_expr) = arg else {
                    continue;
                };

                // Check if this argument has a list type
                if let Some(Type::List(_)) = arg_expr.infer_output_type(ctx) {
                    violations.push((
                        Detection::from_global_span(
                            format!(
                                "List '{}' passed to external command without spread",
                                ctx.span_text(arg_expr.span)
                            ),
                            arg_expr.span,
                        )
                        .with_primary_label(
                            "use `...` to spread list elements as separate arguments",
                        ),
                        arg_expr.span,
                    ));
                }
            }

            violations
        })
    }

    fn fix(&self, context: &LintContext, span: &Self::FixInput<'_>) -> Option<Fix> {
        let var_text = context.span_text(*span);
        Some(Fix {
            explanation: "Add spread operator".into(),
            replacements: vec![Replacement::new(*span, format!("...{var_text}"))],
        })
    }
}

pub static RULE: &dyn Rule = &SpreadListToExternal;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
