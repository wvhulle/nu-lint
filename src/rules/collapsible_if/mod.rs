use nu_protocol::{
    Span,
    ast::{Block, Call, Expr},
};

use crate::{
    Fix, LintLevel, Replacement,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn get_single_if_call<'a>(block: &'a Block, context: &LintContext) -> Option<&'a Call> {
    let pipeline = (block.pipelines.len() == 1).then(|| block.pipelines.first())??;

    let element = (pipeline.elements.len() == 1).then(|| pipeline.elements.first())??;

    match &element.expr.expr {
        Expr::Call(call) if call.is_call_to_command("if", context) => Some(call),
        _ => None,
    }
}

fn get_nested_single_if<'a>(call: &Call, context: &'a LintContext<'a>) -> Option<&'a Call> {
    let then_block = call.get_positional_arg(1)?;
    let then_block_id = then_block.extract_block_id()?;
    get_single_if_call(context.working_set.get_block(then_block_id), context)
}

pub struct FixData {
    replace_span: Span,
    outer_condition: Span,
    inner_condition: Span,
    inner_body: Span,
}

struct CollapsibleIf;

impl DetectFix for CollapsibleIf {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "collapsible_if"
    }

    fn short_description(&self) -> &'static str {
        "Nested if-statements collapsible with `and`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/control_flow.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| match &expr.expr {
            Expr::Call(call) if call.is_call_to_command("if", ctx) => {
                if call.get_else_branch().is_some() {
                    return vec![];
                }

                let Some(inner_call) = get_nested_single_if(call, ctx) else {
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
                );

                let fix_data = FixData {
                    replace_span: call.span(),
                    outer_condition: outer_condition.span,
                    inner_condition: inner_condition.span,
                    inner_body: inner_body.span,
                };

                vec![(detected, fix_data)]
            }
            _ => vec![],
        })
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let outer_cond = context.span_text(fix_data.outer_condition).trim();
        let inner_cond = context.span_text(fix_data.inner_condition).trim();
        let body = context.span_text(fix_data.inner_body).trim();

        let fix_text = format!("if {outer_cond} and {inner_cond} {body}");

        Some(Fix {
            explanation: "Collapse nested if statements".into(),
            replacements: vec![Replacement::new(fix_data.replace_span, fix_text)],
        })
    }
}

pub static RULE: &dyn Rule = &CollapsibleIf;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
