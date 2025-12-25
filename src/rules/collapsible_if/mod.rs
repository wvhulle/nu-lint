use nu_protocol::{Span, ast::Expr};

use crate::{
    Fix, LintLevel, Replacement,
    ast::{call::CallExt, span::SpanExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

#[allow(
    clippy::struct_field_names,
    reason = "Names describe their purpose clearly"
)]
pub struct FixData {
    replace_span: Span,
    outer_condition_span: Span,
    inner_condition_span: Span,
    inner_body_span: Span,
}

struct CollapsibleIf;

impl DetectFix for CollapsibleIf {
    type FixInput = FixData;

    fn id(&self) -> &'static str {
        "collapsible_if"
    }

    fn explanation(&self) -> &'static str {
        "Collapse nested if statements without else clauses into a single if with combined \
         conditions"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/control_flow.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
        context.detect_with_fix_data(|expr, ctx| match &expr.expr {
            Expr::Call(call) if call.is_call_to_command("if", ctx) => {
                if call.get_else_branch().is_some() {
                    return vec![];
                }

                let Some(inner_call) = call.get_nested_single_if(ctx) else {
                    return vec![];
                };

                if inner_call.get_else_branch().is_some() {
                    return vec![];
                }

                let Some(outer_condition) = call.get_first_positional_arg() else {
                    return vec![];
                };
                let Some(inner_condition) = inner_call.get_first_positional_arg() else {
                    return vec![];
                };
                let Some(inner_body) = inner_call.get_positional_arg(1) else {
                    return vec![];
                };

                let detected = Detection::from_global_span(
                    "Nested if statement can be collapsed using 'and'",
                    call.span(),
                )
                .with_primary_label("outer if")
                .with_extra_label(
                    "inner if can be merged with outer condition",
                    inner_call.span(),
                )
                .with_help("Combine conditions using 'and' instead of nesting if statements");

                let fix_data = FixData {
                    replace_span: call.span(),
                    outer_condition_span: outer_condition.span,
                    inner_condition_span: inner_condition.span,
                    inner_body_span: inner_body.span,
                };

                vec![(detected, fix_data)]
            }
            _ => vec![],
        })
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput) -> Option<Fix> {
        let outer_cond = fix_data.outer_condition_span.source_code(context).trim();
        let inner_cond = fix_data.inner_condition_span.source_code(context).trim();
        let body = fix_data.inner_body_span.source_code(context).trim();

        let fix_text = format!("if {outer_cond} and {inner_cond} {body}");

        Some(Fix::with_explanation(
            "Collapse nested if statements",
            vec![Replacement::new(fix_data.replace_span, fix_text)],
        ))
    }
}

pub static RULE: &dyn Rule = &CollapsibleIf;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
