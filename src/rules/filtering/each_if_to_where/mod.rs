use nu_protocol::{
    Span,
    ast::{Argument, Call, Expr, Expression, Traverse},
};

use crate::{
    Fix, LintLevel, Replacement,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct FixData {
    each_span: Span,
    condition_span: Span,
}

fn loop_var_from_each(call: &Call, context: &LintContext) -> Option<String> {
    let first_arg = call.get_first_positional_arg()?;
    let block_id = first_arg.extract_block_id()?;

    let block = context.working_set.get_block(block_id);
    let var_id = block.signature.required_positional.first()?.var_id?;

    let var = context.working_set.get_variable(var_id);
    Some(context.plain_text(var.declaration_span).to_string())
}

fn get_if_then_block(call: &Call) -> Option<&Expression> {
    call.arguments.get(1).and_then(|arg| match arg {
        Argument::Positional(expr) | Argument::Unknown(expr) => Some(expr),
        _ => None,
    })
}

const fn if_has_no_else_clause(call: &Call) -> bool {
    call.arguments.len() == 2
}

fn then_block_returns_loop_var(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    let block = context.working_set.get_block(block_id);

    block.pipelines.len() == 1
        && block.pipelines[0].elements.len() == 1
        && block.pipelines[0].elements[0]
            .expr
            .extract_variable_name(context)
            .is_some_and(|name| name == loop_var_name)
}

fn is_filtering_pattern<'a>(
    block_id: nu_protocol::BlockId,
    context: &'a LintContext,
    loop_var_name: &str,
) -> Option<&'a Call> {
    let block = context.working_set.get_block(block_id);

    if block.pipelines.len() != 1 || block.pipelines[0].elements.len() != 1 {
        return None;
    }

    let elem = &block.pipelines[0].elements[0];

    let Expr::Call(call) = &elem.expr.expr else {
        return None;
    };

    if call.get_call_name(context) != "if" {
        return None;
    }

    if !if_has_no_else_clause(call) {
        return None;
    }

    let then_block_expr = get_if_then_block(call)?;

    let Expr::Block(then_block_id) = &then_block_expr.expr else {
        return None;
    };

    then_block_returns_loop_var(*then_block_id, context, loop_var_name).then_some(call)
}

fn extract_each_block_id(call: &Call) -> Option<nu_protocol::BlockId> {
    call.arguments.first().and_then(|arg| match arg {
        Argument::Positional(expr) | Argument::Unknown(expr) => match &expr.expr {
            Expr::Block(id) | Expr::Closure(id) => Some(*id),
            _ => None,
        },
        _ => None,
    })
}

fn check_expression(expr: &Expression, context: &LintContext) -> Vec<(Detection, FixData)> {
    let Expr::Call(each_call) = &expr.expr else {
        return vec![];
    };

    if each_call.get_call_name(context) != "each" {
        return vec![];
    }

    let Some(loop_var_name) = loop_var_from_each(each_call, context) else {
        return vec![];
    };

    let Some(block_id) = extract_each_block_id(each_call) else {
        return vec![];
    };

    let Some(if_call) = is_filtering_pattern(block_id, context, &loop_var_name) else {
        return vec![];
    };

    // Extract the condition from the if call (first argument)
    let Some(condition_expr) = if_call.get_first_positional_arg() else {
        return vec![];
    };

    let block = context.working_set.get_block(block_id);
    let block_span = block.span.unwrap_or(expr.span);

    let detection = Detection::from_global_span(
        "Consider using 'where' for filtering instead of 'each' with 'if'",
        expr.span,
    )
    .with_primary_label("each with if pattern")
    .with_extra_label("filtering logic inside closure", block_span);

    let fix_data = FixData {
        each_span: expr.span,
        condition_span: condition_expr.span,
    };

    vec![(detection, fix_data)]
}

struct WhereInsteadEachThenIf;

impl DetectFix for WhereInsteadEachThenIf {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "each_if_to_where"
    }

    fn short_description(&self) -> &'static str {
        "Use 'where' for filtering instead of 'each' with 'if'"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/where.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut violations = Vec::new();
        context.ast.flat_map(
            context.working_set,
            &|expr| check_expression(expr, context),
            &mut violations,
        );

        violations
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let condition = context.plain_text(fix_data.condition_span);
        let fix_text = format!("where {condition}");

        Some(Fix::with_explanation(
            "Replace 'each' with 'if' pattern with 'where' for cleaner filtering",
            vec![Replacement::new(fix_data.each_span, fix_text)],
        ))
    }
}

pub static RULE: &dyn Rule = &WhereInsteadEachThenIf;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
